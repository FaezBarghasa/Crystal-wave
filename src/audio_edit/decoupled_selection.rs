#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackTarget {
    ArrangementTimeline,
    EditSelectionLoop,
}

/// Decoupled Playhead & Edit Selection state.
/// Allows looping a microscopic slice (e.g. 5ms consonant) while keeping the arrangement
/// playhead parked so the engineer never loses their position in the song.
#[derive(Debug, Clone, PartialEq)]
pub struct DecoupledPlayheadSelection {
    pub parked_playhead_tick: u64,
    pub current_playhead_tick: u64,
    pub selection_track_id: Option<usize>,
    pub selection_start_tick: u64,
    pub selection_end_tick: u64,
    pub is_selection_active: bool,
    pub playback_target: PlaybackTarget,
}

impl Default for DecoupledPlayheadSelection {
    fn default() -> Self {
        Self {
            parked_playhead_tick: 0,
            current_playhead_tick: 0,
            selection_track_id: None,
            selection_start_tick: 0,
            selection_end_tick: 0,
            is_selection_active: false,
            playback_target: PlaybackTarget::ArrangementTimeline,
        }
    }
}

impl DecoupledPlayheadSelection {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn park_playhead_at(&mut self, tick: u64) {
        self.parked_playhead_tick = tick;
        self.current_playhead_tick = tick;
    }

    pub fn set_edit_selection(&mut self, track_id: usize, start_tick: u64, end_tick: u64) {
        self.selection_track_id = Some(track_id);
        self.selection_start_tick = start_tick.min(end_tick);
        self.selection_end_tick = start_tick.max(end_tick);
        self.is_selection_active = self.selection_end_tick > self.selection_start_tick;
    }

    pub fn clear_selection(&mut self) {
        self.is_selection_active = false;
        self.playback_target = PlaybackTarget::ArrangementTimeline;
    }

    /// Engages microscopic loop playback over the edit selection without moving the parked playhead.
    pub fn engage_selection_loop(&mut self) -> Result<(), &'static str> {
        if !self.is_selection_active {
            return Err("NoActiveSelection");
        }
        self.playback_target = PlaybackTarget::EditSelectionLoop;
        self.current_playhead_tick = self.selection_start_tick;
        Ok(())
    }

    /// Advances playback by `delta_ticks`.
    /// When in `EditSelectionLoop`, wraps cleanly within selection boundaries
    /// while `parked_playhead_tick` remains frozen.
    pub fn advance_playback(&mut self, delta_ticks: u64) -> u64 {
        match self.playback_target {
            PlaybackTarget::ArrangementTimeline => {
                self.current_playhead_tick += delta_ticks;
                self.parked_playhead_tick = self.current_playhead_tick;
                self.current_playhead_tick
            }
            PlaybackTarget::EditSelectionLoop => {
                let sel_len = self.selection_end_tick - self.selection_start_tick;
                if sel_len == 0 {
                    return self.selection_start_tick;
                }
                let offset = (self.current_playhead_tick - self.selection_start_tick + delta_ticks) % sel_len;
                self.current_playhead_tick = self.selection_start_tick + offset;
                self.current_playhead_tick
            }
        }
    }

    /// Restores the timeline back to the original arrangement position.
    pub fn restore_parked_playhead(&mut self) {
        self.playback_target = PlaybackTarget::ArrangementTimeline;
        self.current_playhead_tick = self.parked_playhead_tick;
    }
}
