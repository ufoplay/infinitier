//! MVE / BIK / WBM movies: the video, fitted to the area, with its sound,
//! streamed through a decoder thread, audio and video queues and an A/V
//! clock.

use std::collections::VecDeque;
use std::num::NonZero;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use infinitier_core::imported_resource::movie::{
    MovieDecoder, MovieFormat, MovieSource, MovieVideoFrame,
};
use log::error;
use rodio::{ChannelCount, SampleRate, Source};
use slint::Image;

use super::audio::{Audio, init_audio};
use super::image::rgba_image;
use super::info::format_duration;

/// The queues' locks are only poisoned if a thread panicked holding one.
const POISONED: &str = "a movie thread panicked holding a playback queue";

/// The shortest tick: no faster than a 60 Hz screen.
const MIN_TICK: Duration = Duration::from_millis(16);

/// Target prebuffer for the producer thread, in microseconds. Three
/// seconds of video frames + audio samples are kept in front of the
/// consumer at all times.
const PREBUFFER_US: u64 = 3_000_000;

/// Worst-case audio capacity (3 s @ 48 kHz stereo). Allocated up front;
/// the actual sample rate is detected from the first audio chunk.
const AUDIO_CAPACITY: usize = 3 * 48_000 * 2;

/// How long to wait for the producer thread to surface the file's
/// audio format (channels + sample rate) before giving up and playing
/// without sound. Both MVE and BIK files in the wild always emit audio
/// metadata within the first frame or two, so 250 ms is plenty.
const AUDIO_FMT_TIMEOUT_MS: u64 = 250;

/// Streaming movie player.
///
/// On Play, the viewer:
/// 1. Opens a fresh [`MovieDecoder`] from the saved [`MovieSource`].
///    The decoder dispatches MVE vs. BIK based on the file's magic.
/// 2. Spawns a producer thread that feeds two queues — interleaved
///    `f32` audio samples and decoded RGBA video frames — pre-buffered
///    to ~3 seconds.
/// 3. Attaches a [`MovieAudioSource`] to the rodio sink which pulls
///    audio from the audio queue, and on every tick the viewer pulls
///    the next due frame from the video queue.
///
/// Stop signals the thread, drains the rodio sink, joins the thread
/// (so the decoder is fully released), and clears the frame. The
/// `Drop` impl does the same so threads don't survive viewer changes.
pub struct MovieViewer {
    source: MovieSource,
    /// Container format detected at metadata-probe time. Surfaced in
    /// the bottom info bar; `None` when the file failed to open and
    /// the format never got identified.
    format: Option<MovieFormat>,
    width: u16,
    height: u16,
    frame_duration_us: u32,
    /// Total stream duration captured at metadata-probe time.
    total_duration: Duration,
    /// Fallback for the bottom info bar when the file can't be opened
    /// for metadata.
    decode_error: Option<String>,

    audio: Option<Audio>,
    playback: Option<Playback>,
    /// The most recently shown video frame.
    frame: Option<Image>,
}

impl MovieViewer {
    pub fn new(source: MovieSource) -> Self {
        // Open once just to extract metadata for the info bar. Drop the
        // decoder immediately — actual playback opens a fresh one.
        // Both per-format decoders surface dimensions, frame duration
        // and total length without needing any frame to be pulled
        // first (BIK reads the header; MVE does a one-shot metadata
        // scan in its constructor), so `info()` is fully populated as
        // soon as `open` returns.
        let (format, width, height, frame_duration_us, total_duration, decode_error) =
            match source.open() {
                Ok(dec) => {
                    let info = dec.info();
                    (
                        Some(dec.format()),
                        info.width,
                        info.height,
                        info.frame_duration_us,
                        info.total_duration_us,
                        None,
                    )
                }
                Err(e) => (None, 0, 0, 0, 0, Some(format!("failed to open movie: {e}"))),
            };

        Self {
            source,
            format,
            width,
            height,
            frame_duration_us,
            total_duration: Duration::from_micros(total_duration),
            decode_error,
            audio: init_audio(),
            playback: None,
            frame: None,
        }
    }

    fn start_playback(&mut self) {
        if self.playback.is_some() {
            return;
        }
        if self.frame_duration_us == 0 {
            return;
        }

        let decoder = match self.source.open() {
            Ok(d) => d,
            Err(e) => {
                self.decode_error = Some(format!("failed to open movie: {e}"));
                return;
            }
        };

        // Three seconds worth of video frames, with a small floor so
        // very short clips still buffer ahead.
        let video_capacity = ((PREBUFFER_US / self.frame_duration_us as u64) as usize).max(8);

        let state = Arc::new(PlaybackState::new(AUDIO_CAPACITY, video_capacity));

        let handle = {
            let state = Arc::clone(&state);
            thread::Builder::new()
                .name("movie-viewer-decoder".into())
                .spawn(move || decoder_loop(state, decoder))
                .expect("failed to spawn movie decoder thread")
        };

        // Wire audio if the producer surfaces a format quickly. If it
        // doesn't (no audio in the first frames, or stop arrived early),
        // fall through and play silent video.
        if let Some(audio) = self.audio.as_ref()
            && let Some((channels, sample_rate)) =
                state.wait_for_audio_fmt(Duration::from_millis(AUDIO_FMT_TIMEOUT_MS))
        {
            audio.sink.pause();
            audio.sink.append(MovieAudioSource {
                state: Arc::clone(&state),
                channels,
                sample_rate,
            });
            // Brief warm-up so the very first samples aren't silence.
            for _ in 0..50 {
                if !state.audio_queue.lock().expect(POISONED).is_empty() {
                    break;
                }
                thread::sleep(Duration::from_millis(2));
            }
            audio.sink.play();
        }

        self.playback = Some(Playback {
            state,
            handle: Some(handle),
            epoch: Instant::now(),
            paused_for: Duration::ZERO,
            pause_started: None,
            frames_consumed: 0,
        });
    }

    pub fn stop_playback(&mut self) {
        let Some(mut playback) = self.playback.take() else {
            return;
        };
        playback.state.signal_stop();
        if let Some(audio) = self.audio.as_ref() {
            audio.sink.stop();
        }
        if let Some(handle) = playback.handle.take()
            && let Err(e) = handle.join()
        {
            error!("movie decoder thread panicked: {e:?}");
        }
        self.frame = None;
    }

    fn pause(&mut self) {
        if let Some(playback) = self.playback.as_mut()
            && playback.pause_started.is_none()
        {
            playback.pause_started = Some(Instant::now());
        }
        if let Some(audio) = self.audio.as_ref() {
            audio.sink.pause();
        }
    }

    fn resume(&mut self) {
        if let Some(playback) = self.playback.as_mut()
            && let Some(pause_started) = playback.pause_started.take()
        {
            playback.paused_for += Instant::now().duration_since(pause_started);
        }
        if let Some(audio) = self.audio.as_ref() {
            audio.sink.play();
        }
    }

    fn is_paused(&self) -> bool {
        self.playback
            .as_ref()
            .map(|p| p.pause_started.is_some())
            .unwrap_or(false)
    }

    fn is_playing(&self) -> bool {
        self.playback.is_some() && !self.is_paused()
    }

    fn playback_position(&self) -> Duration {
        let Some(playback) = self.playback.as_ref() else {
            return Duration::ZERO;
        };
        let now = Instant::now();
        let mut elapsed = now
            .duration_since(playback.epoch)
            .saturating_sub(playback.paused_for);
        if let Some(pause_started) = playback.pause_started {
            elapsed = elapsed.saturating_sub(now.duration_since(pause_started));
        }
        elapsed
    }

    /// Pop video frames whose presentation time has passed and show the
    /// latest one (several are popped if the UI was lagging).
    fn advance_frames(&mut self) {
        let Some(playback) = self.playback.as_mut() else {
            return;
        };
        let frame_dur_us = self.frame_duration_us as u64;
        if frame_dur_us == 0 {
            return;
        }

        // Inline `playback_position` here so we can keep the `&mut
        // Playback` borrow alive while we compute it.
        let now = Instant::now();
        let mut elapsed = now
            .duration_since(playback.epoch)
            .saturating_sub(playback.paused_for);
        if let Some(pause_started) = playback.pause_started {
            elapsed = elapsed.saturating_sub(now.duration_since(pause_started));
        }
        let target = elapsed.as_micros() as u64 / frame_dur_us;

        let mut latest: Option<MovieVideoFrame> = None;
        {
            let mut q = playback.state.video_queue.lock().expect(POISONED);
            while playback.frames_consumed <= target {
                match q.pop_front() {
                    Some(frame) => {
                        latest = Some(frame);
                        playback.frames_consumed += 1;
                    }
                    None => break,
                }
            }
        }
        playback.state.video_cond.notify_all();

        if let Some(frame) = latest {
            self.frame = Some(rgba_image(
                &frame.pixels,
                u32::from(frame.width),
                u32::from(frame.height),
            ));
        }
    }

    /// Detect natural EOS: producer has flagged it AND both queues are
    /// drained, AND audio sink (if any) is empty. Then teardown.
    fn poll_for_eos(&mut self) {
        let Some(playback) = self.playback.as_ref() else {
            return;
        };
        let producer_done = playback.state.eos.load(Ordering::Acquire);
        if !producer_done {
            return;
        }
        let video_drained = playback
            .state
            .video_queue
            .lock()
            .expect(POISONED)
            .is_empty();
        let audio_drained = self.audio.as_ref().map(|a| a.sink.empty()).unwrap_or(true);
        if video_drained && audio_drained {
            self.stop_playback();
        }
    }
}

impl Drop for MovieViewer {
    fn drop(&mut self) {
        // Make sure the producer thread is fully gone before the
        // viewer's heap allocations go away.
        self.stop_playback();
    }
}

// ─── Producer thread + shared state ───────────────────────────────────────────

/// Active playback session. Dropped on Stop and on Drop of the viewer.
struct Playback {
    state: Arc<PlaybackState>,
    handle: Option<thread::JoinHandle<()>>,
    /// Wall-clock instant that playback (re)started at.
    epoch: Instant,
    /// Total time spent paused in this session, accumulated each Resume.
    paused_for: Duration,
    /// If currently paused, when the pause began. `None` when running.
    pause_started: Option<Instant>,
    /// How many video frames the UI has popped from the queue. Used to
    /// decide which frames are still due.
    frames_consumed: u64,
}

/// Bounded shared state between the producer thread and the UI / audio
/// callback thread.
struct PlaybackState {
    audio_queue: Mutex<VecDeque<f32>>,
    audio_cond: Condvar,
    audio_capacity: usize,

    video_queue: Mutex<VecDeque<MovieVideoFrame>>,
    video_cond: Condvar,
    video_capacity: usize,

    /// Audio format (channels, sample_rate) extracted from the first
    /// audio chunk. `None` until the producer sees one — the UI waits
    /// briefly on `audio_fmt_cond` so the rodio source is constructed
    /// with the right rate.
    audio_fmt: Mutex<Option<(NonZero<u16>, NonZero<u32>)>>,
    audio_fmt_cond: Condvar,

    /// Producer reached natural end-of-stream (or hit an error).
    eos: AtomicBool,
    /// UI thread asked the producer to exit ASAP.
    stop: AtomicBool,
}

impl PlaybackState {
    fn new(audio_capacity: usize, video_capacity: usize) -> Self {
        Self {
            audio_queue: Mutex::new(VecDeque::with_capacity(audio_capacity)),
            audio_cond: Condvar::new(),
            audio_capacity,
            video_queue: Mutex::new(VecDeque::with_capacity(video_capacity)),
            video_cond: Condvar::new(),
            video_capacity,
            audio_fmt: Mutex::new(None),
            audio_fmt_cond: Condvar::new(),
            eos: AtomicBool::new(false),
            stop: AtomicBool::new(false),
        }
    }

    fn signal_stop(&self) {
        self.stop.store(true, Ordering::Release);
        // Wake every parked waiter (producer parked on a full queue,
        // UI parked on the audio fmt).
        self.audio_cond.notify_all();
        self.video_cond.notify_all();
        self.audio_fmt_cond.notify_all();
    }

    /// Wait up to `timeout` for the producer to emit an audio format.
    /// Returns `None` on timeout / EOS-without-audio / stop.
    fn wait_for_audio_fmt(&self, timeout: Duration) -> Option<(NonZero<u16>, NonZero<u32>)> {
        let q = self.audio_fmt.lock().expect(POISONED);
        let (q, _) = self
            .audio_fmt_cond
            .wait_timeout_while(q, timeout, |fmt| {
                fmt.is_none()
                    && !self.eos.load(Ordering::Acquire)
                    && !self.stop.load(Ordering::Acquire)
            })
            .expect(POISONED);
        *q
    }
}

fn decoder_loop(state: Arc<PlaybackState>, mut decoder: MovieDecoder) {
    loop {
        if state.stop.load(Ordering::Acquire) {
            return;
        }

        // Park until there's room for another video frame. The audio
        // queue's own backpressure is handled inside the audio push
        // below.
        {
            let mut q = state.video_queue.lock().expect(POISONED);
            while q.len() >= state.video_capacity && !state.stop.load(Ordering::Acquire) {
                q = state.video_cond.wait(q).expect(POISONED);
            }
            if state.stop.load(Ordering::Acquire) {
                return;
            }
        }

        let frame = match decoder.next_frame() {
            Ok(Some(f)) => f,
            Ok(None) => {
                state.eos.store(true, Ordering::Release);
                state.audio_cond.notify_all();
                state.video_cond.notify_all();
                state.audio_fmt_cond.notify_all();
                return;
            }
            Err(e) => {
                error!("[movie] decode error: {e}");
                state.eos.store(true, Ordering::Release);
                state.audio_cond.notify_all();
                state.video_cond.notify_all();
                state.audio_fmt_cond.notify_all();
                return;
            }
        };

        for chunk in &frame.audio {
            // Surface the audio format the first time we see one.
            {
                let mut fmt = state.audio_fmt.lock().expect(POISONED);
                if fmt.is_none()
                    && let Some(c) = NonZero::new(chunk.channels as u16)
                    && let Some(r) = NonZero::new(chunk.sample_rate)
                {
                    *fmt = Some((c, r));
                    state.audio_fmt_cond.notify_all();
                }
            }

            let mut q = state.audio_queue.lock().expect(POISONED);
            while q.len() + chunk.samples.len() > state.audio_capacity
                && !state.stop.load(Ordering::Acquire)
            {
                q = state.audio_cond.wait(q).expect(POISONED);
            }
            if state.stop.load(Ordering::Acquire) {
                return;
            }
            for &s in &chunk.samples {
                q.push_back(s as f32 / 32768.0);
            }
            state.audio_cond.notify_all();
        }

        let mut q = state.video_queue.lock().expect(POISONED);
        q.push_back(frame.video);
        state.video_cond.notify_all();
    }
}

// ─── rodio Source pulling audio out of the playback state ─────────────────────

struct MovieAudioSource {
    state: Arc<PlaybackState>,
    channels: ChannelCount,
    sample_rate: SampleRate,
}

impl Iterator for MovieAudioSource {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        let q = self.state.audio_queue.lock().expect(POISONED);
        let mut q = q;
        if let Some(s) = q.pop_front() {
            self.state.audio_cond.notify_all();
            return Some(s);
        }
        if self.state.eos.load(Ordering::Acquire) {
            return None;
        }
        // Underrun: brief wait so we don't immediately inject silence.
        let (mut q, _) = self
            .state
            .audio_cond
            .wait_timeout(q, Duration::from_millis(5))
            .expect(POISONED);
        if let Some(s) = q.pop_front() {
            self.state.audio_cond.notify_all();
            return Some(s);
        }
        if self.state.eos.load(Ordering::Acquire) {
            return None;
        }
        Some(0.0)
    }
}

impl Source for MovieAudioSource {
    fn current_span_len(&self) -> Option<usize> {
        None
    }
    fn channels(&self) -> ChannelCount {
        self.channels
    }
    fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

impl MovieViewer {
    /// The frame on screen, while playing.
    pub fn frame(&self) -> Option<&Image> {
        self.frame.as_ref()
    }

    pub fn size(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    pub fn decode_error(&self) -> Option<&str> {
        self.decode_error.as_deref()
    }

    /// The movie opened: Play and Stop work.
    pub fn can_play(&self) -> bool {
        self.frame_duration_us > 0 && self.decode_error.is_none()
    }

    pub fn playing(&self) -> bool {
        self.is_playing()
    }

    /// How far playback is, from 0 to 1.
    pub fn progress(&self) -> f32 {
        let total = self.total_duration.as_secs_f64();
        if total > 0.0 {
            (self.playback_position().as_secs_f64() / total).clamp(0.0, 1.0) as f32
        } else {
            0.0
        }
    }

    /// `"position / duration"`.
    pub fn time_text(&self) -> String {
        format!(
            "{} / {}",
            format_duration(self.playback_position()),
            format_duration(self.total_duration)
        )
    }

    pub fn toggle_play_pause(&mut self) {
        if self.playback.is_none() {
            self.start_playback();
        } else if self.is_paused() {
            self.resume();
        } else {
            self.pause();
        }
    }

    /// How often [`tick`](Self::tick) wants to run: once a frame while a
    /// playback is alive, paused too (the queues keep filling).
    pub fn tick_interval(&self) -> Option<Duration> {
        self.playback
            .as_ref()
            .map(|_| Duration::from_micros(u64::from(self.frame_duration_us)).max(MIN_TICK))
    }

    /// Show the frame that is due, and notice the end of the movie.
    pub fn tick(&mut self) {
        self.advance_frames();
        self.poll_for_eos();
    }

    pub fn info(&self) -> Vec<String> {
        let mut info = vec![self.source.name.clone()];
        if let Some(format) = self.format {
            info.push(format.to_string());
        }
        let fps = if self.frame_duration_us > 0 {
            1_000_000.0 / f64::from(self.frame_duration_us)
        } else {
            0.0
        };
        info.extend([
            format!("{}×{}", self.width, self.height),
            format!("{fps:.2} fps"),
            format_duration(self.total_duration),
        ]);
        info
    }
}
