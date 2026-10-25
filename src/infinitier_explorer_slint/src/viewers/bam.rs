//! BAM animations: a cycle picker, a frame slider and a looping player.

use std::time::{Duration, Instant};

use infinitier_core::{
    game::GameResource,
    imported_resource::bam::ImportedBam,
    resource::bam::{BamV1, Type},
};
use slint::Image;

use super::image::rgba_image;
use super::info::{origin_label, size_label};

/// The selected frame, composited on its cycle's canvas.
pub struct Frame {
    pub image: Image,
    pub width: u32,
    pub height: u32,
}

pub struct BamViewer {
    bam: ImportedBam,
    selected_cycle: usize,
    selected_frame_in_cycle: usize,
    /// (cycle, frame_in_cycle) currently rendered into `frame`.
    rendered: Option<(usize, usize)>,
    frame: Option<Frame>,
    /// `Some` while looping; carries the wall-clock anchor used to derive
    /// which frame is due.
    playback: Option<Playback>,
}

struct Playback {
    /// Wall-clock instant of the anchor.
    epoch: Instant,
    /// `selected_frame_in_cycle` at `epoch`. Later frame indices are
    /// `(anchor_frame + elapsed / BamV1::DEFAULT_FRAME_DURATION) % len`.
    anchor_frame: usize,
}

impl BamViewer {
    pub fn new(bam: ImportedBam) -> Self {
        let mut viewer = Self {
            bam,
            selected_cycle: 0,
            selected_frame_in_cycle: 0,
            rendered: None,
            frame: None,
            playback: None,
        };
        viewer.refresh_frame();
        viewer
    }

    /// The current (cycle, frame_in_cycle) selection as an index into
    /// `bam.frames`.
    pub fn current_global_frame_index(&self) -> Option<usize> {
        let cycle = self.bam.cycles.get(self.selected_cycle)?;
        cycle
            .frame_indices
            .get(self.selected_frame_in_cycle)
            .copied()
    }

    /// Re-render the current frame if the selection changed since the last
    /// render. The frame is composited into the cycle's shared canvas so its
    /// anchor stays pinned across frames.
    fn refresh_frame(&mut self) {
        let key = (self.selected_cycle, self.selected_frame_in_cycle);
        if self.rendered == Some(key) {
            return;
        }
        self.rendered = Some(key);
        self.frame = self
            .bam
            .render_frame_centered(self.selected_cycle, self.selected_frame_in_cycle)
            .map(|image| Frame {
                image: rgba_image(image.as_raw(), image.width(), image.height()),
                width: image.width(),
                height: image.height(),
            });
    }

    pub fn frame(&self) -> Option<&Frame> {
        self.frame.as_ref()
    }

    pub fn frames_in_selected_cycle(&self) -> usize {
        self.bam
            .cycles
            .get(self.selected_cycle)
            .map(|cycle| cycle.frame_indices.len())
            .unwrap_or(0)
    }

    pub fn selected_cycle(&self) -> Option<usize> {
        (self.selected_cycle < self.bam.cycles.len()).then_some(self.selected_cycle)
    }

    pub fn selected_frame(&self) -> usize {
        self.selected_frame_in_cycle
    }

    /// `"{i} ({n} frames)"` per cycle.
    pub fn cycle_labels(&self) -> Vec<String> {
        self.bam
            .cycles
            .iter()
            .enumerate()
            .map(|(i, cycle)| format!("{i} ({} frames)", cycle.frame_indices.len()))
            .collect()
    }

    pub fn is_playing(&self) -> bool {
        self.playback.is_some()
    }

    pub fn can_play(&self) -> bool {
        self.frames_in_selected_cycle() > 1
    }

    pub fn toggle_play(&mut self) {
        if self.playback.is_some() {
            self.playback = None;
        } else if self.can_play() {
            self.playback = Some(Playback {
                epoch: Instant::now(),
                anchor_frame: self.selected_frame_in_cycle,
            });
        }
    }

    pub fn select_cycle(&mut self, cycle: usize) {
        let Some(len) = self.bam.cycles.get(cycle).map(|c| c.frame_indices.len()) else {
            return;
        };
        self.selected_cycle = cycle;
        self.selected_frame_in_cycle = self.selected_frame_in_cycle.min(len.saturating_sub(1));
        if len < 2 {
            // Nothing left to loop through.
            self.playback = None;
        }
        self.rebase_playback();
        self.refresh_frame();
    }

    pub fn select_frame(&mut self, frame: usize) {
        let len = self.frames_in_selected_cycle();
        if len == 0 {
            return;
        }
        self.selected_frame_in_cycle = frame.min(len - 1);
        self.rebase_playback();
        self.refresh_frame();
    }

    /// Anchor the playback clock to the current selection, so playback
    /// resumes from where the user left it instead of jumping.
    fn rebase_playback(&mut self) {
        if let Some(playback) = self.playback.as_mut() {
            playback.epoch = Instant::now();
            playback.anchor_frame = self.selected_frame_in_cycle;
        }
    }

    /// How often [`tick`](Self::tick) wants to run: once a frame, while
    /// playing.
    pub fn tick_interval(&self) -> Option<Duration> {
        self.playback
            .as_ref()
            .map(|_| BamV1::DEFAULT_FRAME_DURATION)
    }

    /// Advance to the frame due by the playback clock.
    pub fn tick(&mut self) {
        let Some(playback) = self.playback.as_ref() else {
            return;
        };
        let len = self.frames_in_selected_cycle();
        if len == 0 {
            return;
        }
        let elapsed = playback.epoch.elapsed().as_nanos();
        let ticks = elapsed / BamV1::DEFAULT_FRAME_DURATION.as_nanos();
        self.selected_frame_in_cycle = (playback.anchor_frame + ticks as usize) % len;
        self.refresh_frame();
    }

    pub fn info(&self, resource: &GameResource) -> Vec<String> {
        let current = self
            .current_global_frame_index()
            .and_then(|i| self.bam.frames.get(i));
        let (width, height, center_x, center_y) = match current {
            Some(f) => (f.width, f.height, f.center_x, f.center_y),
            None => (0, 0, 0, 0),
        };
        let bam_type = match self.bam.bam_type {
            Type::BamV1 => "BAM V1",
            Type::BamV2 => "BAM V2",
            Type::BamC => "BAMC",
        };
        vec![
            format!("{width} × {height} px"),
            format!("center ({center_x}, {center_y})"),
            size_label(resource),
            bam_type.to_owned(),
            format!("{} frames", self.bam.frames.len()),
            format!("{} cycles", self.bam.cycles.len()),
            origin_label(resource),
        ]
    }
}
