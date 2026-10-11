use crate::fl_studio::automation_clip::AutomationClip;
use crate::fl_studio::piano_roll::FlPianoRollNote;

/// Independent Pattern containing multi-instrument MIDI data
#[derive(Debug, Clone)]
pub struct FlPattern {
    pub id: usize,
    pub name: String,
    pub notes: Vec<FlPianoRollNote>,
    pub color: (u8, u8, u8),
}

impl FlPattern {
    pub fn new(id: usize, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            notes: Vec::new(),
            color: (0x4a, 0x90, 0xe2),
        }
    }

    pub fn add_note(&mut self, note: FlPianoRollNote) {
        self.notes.push(note);
    }
}

/// Coexisting Playlist Items (Audio, MIDI Pattern, and Automation in any track lane)
#[derive(Debug, Clone)]
pub enum PlaylistItem {
    PatternInstance {
        pattern_id: usize,
        start_tick: u64,
        length_ticks: u64,
    },
    AudioClipInstance {
        sample_path: String,
        start_tick: u64,
        length_ticks: u64,
        gain_db: f32,
    },
    AutomationClipInstance {
        clip_id: usize,
        start_tick: u64,
        length_ticks: u64,
    },
}

impl PlaylistItem {
    pub fn start_tick(&self) -> u64 {
        match self {
            Self::PatternInstance { start_tick, .. } => *start_tick,
            Self::AudioClipInstance { start_tick, .. } => *start_tick,
            Self::AutomationClipInstance { start_tick, .. } => *start_tick,
        }
    }

    pub fn length_ticks(&self) -> u64 {
        match self {
            Self::PatternInstance { length_ticks, .. } => *length_ticks,
            Self::AudioClipInstance { length_ticks, .. } => *length_ticks,
            Self::AutomationClipInstance { length_ticks, .. } => *length_ticks,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct PlaylistTrackLane {
    pub id: usize,
    pub name: String,
    pub items: Vec<PlaylistItem>,
    pub muted: bool,
    pub solo: bool,
}

pub struct FlPlaylist {
    pub lanes: Vec<PlaylistTrackLane>,
    pub patterns: Vec<FlPattern>,
    pub automation_clips: Vec<AutomationClip>,
}

impl FlPlaylist {
    pub fn new(num_lanes: usize) -> Self {
        let mut lanes = Vec::with_capacity(num_lanes);
        for id in 0..num_lanes {
            lanes.push(PlaylistTrackLane {
                id,
                name: format!("Track {}", id + 1),
                items: Vec::new(),
                muted: false,
                solo: false,
            });
        }
        Self {
            lanes,
            patterns: Vec::new(),
            automation_clips: Vec::new(),
        }
    }

    pub fn create_pattern(&mut self, name: impl Into<String>) -> usize {
        let id = self.patterns.len();
        self.patterns.push(FlPattern::new(id, name));
        id
    }

    pub fn stamp_item(&mut self, lane_id: usize, item: PlaylistItem) -> bool {
        if let Some(lane) = self.lanes.get_mut(lane_id) {
            lane.items.push(item);
            true
        } else {
            false
        }
    }
}
