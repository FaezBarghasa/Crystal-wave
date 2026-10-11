/// A ghost note projected from another track into the active Piano Roll
#[derive(Debug, Clone, PartialEq)]
pub struct GhostNote {
    pub source_track_id: usize,
    pub pitch: u8,
    pub start_tick: u64,
    pub length_ticks: u64,
    pub velocity: u8,
    pub is_slide: bool,
    pub microtonal_cents: f32, // +/- 100 cents microtonal detune
    pub color_rgb: [u8; 3],
}

impl GhostNote {
    pub fn new(
        source_track_id: usize,
        pitch: u8,
        start_tick: u64,
        length_ticks: u64,
        velocity: u8,
        color_rgb: [u8; 3],
    ) -> Self {
        Self {
            source_track_id,
            pitch: pitch.clamp(0, 127),
            start_tick,
            length_ticks,
            velocity: velocity.clamp(1, 127),
            is_slide: false,
            microtonal_cents: 0.0,
            color_rgb,
        }
    }

    pub fn with_slide(mut self, is_slide: bool) -> Self {
        self.is_slide = is_slide;
        self
    }

    pub fn with_microtonal(mut self, cents: f32) -> Self {
        self.microtonal_cents = cents.clamp(-100.0, 100.0);
        self
    }

    pub fn end_tick(&self) -> u64 {
        self.start_tick + self.length_ticks
    }
}

/// Context-Aware Piano Roll Ghost Polyphony projector
#[derive(Debug, Clone, Default, PartialEq)]
pub struct GhostPolyphonyView {
    pub active_track_id: usize,
    pub ghost_notes: Vec<GhostNote>,
    pub ghost_opacity: f32, // 0.0 to 1.0 (default 0.35)
}

pub type RawGhostNoteTuple = (usize, u8, u64, u64, u8, [u8; 3]);

impl GhostPolyphonyView {
    pub fn new(active_track_id: usize) -> Self {
        Self {
            active_track_id,
            ghost_notes: Vec::new(),
            ghost_opacity: 0.35,
        }
    }

    /// Clears and repopulates ghost notes from all non-active tracks in the arrangement.
    pub fn update_ghosts(&mut self, all_track_notes: &[RawGhostNoteTuple]) {
        self.ghost_notes.clear();
        for &(track_id, pitch, start, len, vel, color) in all_track_notes {
            if track_id != self.active_track_id {
                self.ghost_notes.push(GhostNote::new(
                    track_id, pitch, start, len, vel, color,
                ));
            }
        }
    }

    /// Queries all ghost notes active at a specific tick position to check for harmonic clashes.
    pub fn active_ghosts_at(&self, tick: u64) -> Vec<&GhostNote> {
        self.ghost_notes
            .iter()
            .filter(|n| tick >= n.start_tick && tick < n.end_tick())
            .collect()
    }
}
