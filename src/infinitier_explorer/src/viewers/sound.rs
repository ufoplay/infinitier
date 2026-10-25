//! ACM / WAV / WAVC sounds: a player streaming the decoded clip through a
//! decoder thread, a bounded queue and a rodio source.

use std::collections::VecDeque;
use std::num::NonZero;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;

use infinitier_core::imported_resource::sound::{SoundDecoder, SoundFormat, SoundInfo};
use log::error;
use rodio::{ChannelCount, SampleRate, Source};

use super::audio::{Audio, init_audio};
use super::info::format_duration;

/// How often the player refreshes its position while playing.
const TICK: Duration = Duration::from_millis(100);

/// The queue's lock is only poisoned if a thread panicked holding it.
const POISONED: &str = "a sound thread panicked holding the audio queue";

/// How many `i16` samples the decoder thread reads in one `read_samples`
/// call. A small chunk keeps lock-hold time short and lets the producer
/// react to a Stop request promptly.
const CHUNK_SAMPLES: usize = 2048;

/// Player for any [`SoundDecoder`] — i.e. ACM, RIFF/WAVE, or WAVC.
///
/// Decoding runs on a dedicated background thread that streams samples
/// in `CHUNK_SAMPLES`-sized blocks via `SoundDecoder::read_samples` and
/// pushes them into a bounded shared queue holding ~2 seconds of audio.
/// The rodio source pulls from that queue on the audio callback thread,
/// so the full clip is never resident in memory.
///
/// On Stop and on Drop the thread is signalled to exit and joined so
/// the decoder can be reused (Stop) or fully released (Drop).
pub struct SoundViewer {
    info: SoundInfo,
    name: String,
    format: SoundFormat,
    duration: Duration,
    /// Holds the decoder when not playing. Moved into the producer
    /// thread on Play, returned via the `JoinHandle` when the thread
    /// exits, then stashed back here (after a `reset`).
    decoder: Option<SoundDecoder>,
    /// Audio output, kept on the viewer so playback persists while it is
    /// shown. `None` if the system has no usable audio device.
    audio: Option<Audio>,
    /// Active producer thread + shared buffer. `None` when not playing.
    playback: Option<Playback>,
}

impl SoundViewer {
    pub fn new(decoder: SoundDecoder) -> Self {
        let info = decoder.info();
        let name = decoder.name().to_string();
        let format = decoder.format();
        let duration = compute_duration(&info);
        let audio = init_audio();

        Self {
            info,
            name,
            format,
            duration,
            decoder: Some(decoder),
            audio,
            playback: None,
        }
    }

    /// Spawn the producer thread and wire a streaming source into the
    /// rodio sink. Idempotent — does nothing if already playing.
    fn start_playback(&mut self) {
        if self.playback.is_some() {
            return;
        }
        let Some(audio) = self.audio.as_ref() else {
            return;
        };
        let Some(decoder) = self.decoder.take() else {
            return;
        };
        let Some(channels) = NonZero::new(self.info.channels) else {
            self.decoder = Some(decoder);
            return;
        };
        let Some(sample_rate) = NonZero::new(self.info.sample_rate) else {
            self.decoder = Some(decoder);
            return;
        };

        // 2 seconds of interleaved samples, with a small floor so very
        // low-rate fixtures still get a sensible buffer.
        let capacity = (2 * self.info.sample_rate as usize * self.info.channels as usize).max(8192);
        let buffer = Arc::new(AudioBuffer::new(capacity));

        let handle = {
            let buffer = Arc::clone(&buffer);
            thread::Builder::new()
                .name("sound-viewer-decoder".into())
                .spawn(move || decoder_loop(buffer, decoder))
                .expect("failed to spawn decoder thread")
        };

        let source = StreamingSource {
            buffer: Arc::clone(&buffer),
            channels,
            sample_rate,
            total_duration: self.duration,
        };

        // Append the source first (still paused), give the producer a
        // moment to fill some samples, then unpause — avoids silence at
        // the very start of playback.
        audio.sink.pause();
        audio.sink.append(source);
        for _ in 0..50 {
            if buffer.has_samples() {
                break;
            }
            thread::sleep(Duration::from_millis(2));
        }
        audio.sink.play();

        self.playback = Some(Playback {
            buffer,
            handle: Some(handle),
        });
    }

    /// Tear down the active playback: signal the producer thread to
    /// exit, drain the rodio sink, join the thread, and put the
    /// decoder back into `self.decoder` after rewinding it.
    pub fn stop_playback(&mut self) {
        let Some(mut playback) = self.playback.take() else {
            return;
        };
        playback.buffer.signal_stop();
        if let Some(audio) = self.audio.as_ref() {
            audio.sink.stop();
        }
        if let Some(handle) = playback.handle.take()
            && let Ok(mut decoder) = handle.join()
        {
            if let Err(e) = decoder.reset() {
                error!("decoder reset failed: {e}");
            }
            self.decoder = Some(decoder);
        }
    }

    /// Detect natural end-of-stream — buffer EOS flagged + sink drained
    /// — and reclaim the decoder so the next Play press starts clean.
    fn poll_for_eos(&mut self) {
        let Some(playback) = self.playback.as_mut() else {
            return;
        };
        let sink_empty = self.audio.as_ref().map(|a| a.sink.empty()).unwrap_or(true);
        let thread_done = playback
            .handle
            .as_ref()
            .map(|h| h.is_finished())
            .unwrap_or(true);
        if sink_empty && thread_done {
            // Take the playback out so we can join the handle. `join`
            // here does not block — `is_finished` was true above.
            let Some(mut playback) = self.playback.take() else {
                return;
            };
            if let Some(handle) = playback.handle.take()
                && let Ok(mut decoder) = handle.join()
            {
                if let Err(e) = decoder.reset() {
                    error!("decoder reset failed: {e}");
                }
                self.decoder = Some(decoder);
            }
        }
    }
}

impl Drop for SoundViewer {
    fn drop(&mut self) {
        // Make sure the producer thread is fully gone before the
        // viewer's heap allocations go away.
        self.stop_playback();
    }
}

#[inline]
fn i16_to_f32(s: i16) -> f32 {
    s as f32 / 32768.0
}

fn compute_duration(info: &SoundInfo) -> Duration {
    if info.sample_rate == 0 || info.channels == 0 {
        return Duration::ZERO;
    }
    Duration::from_secs_f64(info.frames() as f64 / info.sample_rate as f64)
}

// ─── Streaming buffer + producer thread ───────────────────────────────────────

/// Bounded SPMC-style queue protected by a `Mutex` + `Condvar`. The
/// producer waits when the queue is full; the consumer (audio source)
/// never waits — it returns silence on underrun and `None` once the
/// producer has flagged end-of-stream.
struct AudioBuffer {
    queue: Mutex<VecDeque<f32>>,
    cond: Condvar,
    capacity: usize,
    /// Producer thread reached natural end-of-stream (or hit an error).
    eos: AtomicBool,
    /// UI thread (Stop or Drop) has asked the producer to exit promptly.
    stop: AtomicBool,
}

impl AudioBuffer {
    fn new(capacity: usize) -> Self {
        Self {
            queue: Mutex::new(VecDeque::with_capacity(capacity)),
            cond: Condvar::new(),
            capacity,
            eos: AtomicBool::new(false),
            stop: AtomicBool::new(false),
        }
    }

    fn signal_stop(&self) {
        self.stop.store(true, Ordering::Release);
        self.cond.notify_all();
    }

    fn has_samples(&self) -> bool {
        !self.queue.lock().expect(POISONED).is_empty()
    }
}

/// Pull samples from `decoder` in `CHUNK_SAMPLES`-sized blocks and push
/// them into `buffer`. Returns the decoder so the UI can rewind it for
/// the next playback.
fn decoder_loop(buffer: Arc<AudioBuffer>, mut decoder: SoundDecoder) -> SoundDecoder {
    let mut chunk = vec![0i16; CHUNK_SAMPLES];
    loop {
        if buffer.stop.load(Ordering::Acquire) {
            return decoder;
        }

        // Park the thread until the queue has room for another chunk.
        {
            let mut q = buffer.queue.lock().expect(POISONED);
            while q.len() + chunk.len() > buffer.capacity && !buffer.stop.load(Ordering::Acquire) {
                q = buffer.cond.wait(q).expect(POISONED);
            }
            if buffer.stop.load(Ordering::Acquire) {
                return decoder;
            }
        }

        let n = match decoder.read_samples(&mut chunk) {
            Ok(0) => {
                buffer.eos.store(true, Ordering::Release);
                buffer.cond.notify_all();
                return decoder;
            }
            Ok(n) => n,
            Err(e) => {
                error!("[{}] decode error: {e}", decoder.name());
                buffer.eos.store(true, Ordering::Release);
                buffer.cond.notify_all();
                return decoder;
            }
        };

        let mut q = buffer.queue.lock().expect(POISONED);
        for &s in &chunk[..n] {
            q.push_back(i16_to_f32(s));
        }
        buffer.cond.notify_all();
    }
}

/// Active playback state held by the viewer while the decoder thread
/// is alive.
struct Playback {
    buffer: Arc<AudioBuffer>,
    handle: Option<thread::JoinHandle<SoundDecoder>>,
}

/// rodio Source pulling samples out of [`AudioBuffer`].
struct StreamingSource {
    buffer: Arc<AudioBuffer>,
    channels: ChannelCount,
    sample_rate: SampleRate,
    total_duration: Duration,
}

impl Iterator for StreamingSource {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        let mut q = self.buffer.queue.lock().expect(POISONED);
        if let Some(s) = q.pop_front() {
            // Wake the producer in case it was parked on a full queue.
            self.buffer.cond.notify_all();
            return Some(s);
        }
        // Empty queue — was that "underrun" or "natural EOS"?
        if self.buffer.eos.load(Ordering::Acquire) {
            return None;
        }
        // Underrun. Briefly wait for samples instead of immediately
        // injecting silence, but never block the audio thread for long.
        let (mut q, _) = self
            .buffer
            .cond
            .wait_timeout(q, Duration::from_millis(5))
            .expect(POISONED);
        if let Some(s) = q.pop_front() {
            self.buffer.cond.notify_all();
            return Some(s);
        }
        if self.buffer.eos.load(Ordering::Acquire) {
            return None;
        }
        Some(0.0)
    }
}

impl Source for StreamingSource {
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
        Some(self.total_duration)
    }
}

impl SoundViewer {
    /// Whether the system has no audio device: the player is disabled.
    pub fn device_missing(&self) -> bool {
        self.audio.is_none()
    }

    /// The decoder or a playback holds the clip: Play and Stop work.
    pub fn can_play(&self) -> bool {
        self.decoder.is_some() || self.playback.is_some()
    }

    pub fn is_playing(&self) -> bool {
        self.playback.is_some()
            && self
                .audio
                .as_ref()
                .map(|a| !a.sink.is_paused() && !a.sink.empty())
                .unwrap_or(false)
    }

    /// Stopped is at the start, even before the sink has dropped the
    /// stopped source (rodio does that on its audio thread).
    fn current_position(&self) -> Duration {
        match self.audio.as_ref() {
            Some(a) if self.playback.is_some() && !a.sink.empty() => a.sink.get_pos(),
            _ => Duration::ZERO,
        }
    }

    /// How far playback is, from 0 to 1.
    pub fn progress(&self) -> f32 {
        let total = self.duration.as_secs_f64();
        if total > 0.0 {
            (self.current_position().as_secs_f64() / total).clamp(0.0, 1.0) as f32
        } else {
            0.0
        }
    }

    /// `"position / duration"`.
    pub fn time_text(&self) -> String {
        format!(
            "{} / {}",
            format_duration(self.current_position()),
            format_duration(self.duration)
        )
    }

    pub fn toggle_play_pause(&mut self) {
        let Some(audio) = self.audio.as_ref() else {
            return;
        };
        if self.playback.is_some() {
            // Already streaming — just toggle the sink. The producer thread
            // keeps filling the buffer (and parks on the condvar when it
            // fills up), so it isn't torn down.
            if audio.sink.is_paused() {
                audio.sink.play();
            } else {
                audio.sink.pause();
            }
        } else {
            // Cold start: spawn the producer + attach a fresh source.
            self.start_playback();
        }
    }

    /// How often [`tick`](Self::tick) wants to run: while a playback is
    /// alive, to move the position and notice the end of the clip.
    pub fn tick_interval(&self) -> Option<Duration> {
        self.playback.as_ref().map(|_| TICK)
    }

    pub fn tick(&mut self) {
        self.poll_for_eos();
    }

    pub fn info(&self) -> Vec<String> {
        vec![
            self.name.clone(),
            self.format.to_string(),
            format!("{} Hz", self.info.sample_rate),
            format!("{} ch", self.info.channels),
            format!("{}-bit", self.info.bits_per_sample),
            format!("{} samples", self.info.frames()),
            format_duration(self.duration),
        ]
    }
}
