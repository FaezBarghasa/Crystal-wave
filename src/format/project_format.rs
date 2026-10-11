use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProjectMetadata {
    pub name: String,
    pub author: String,
    pub sample_rate: f32,
    pub bpm: f32,
    pub time_signature: (u8, u8),
    pub format_version: String,
}

impl Default for ProjectMetadata {
    fn default() -> Self {
        Self {
            name: "Untitled Project".into(),
            author: "User".into(),
            sample_rate: 44100.0,
            bpm: 120.0,
            time_signature: (4, 4),
            format_version: "12.3.1".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SerializedRegion {
    pub id: usize,
    pub track_id: usize,
    pub name: String,
    pub start_tick: u64,
    pub length_ticks: u64,
    pub muted: bool,
    pub gain_db: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SerializedTrack {
    pub id: usize,
    pub name: String,
    pub track_type: String,
    pub volume_fader_db: f32,
    pub pan: f32,
    pub muted: bool,
    pub solo: bool,
    pub regions: Vec<SerializedRegion>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CrystalWaveProject {
    pub metadata: ProjectMetadata,
    pub tracks: Vec<SerializedTrack>,
    pub cycle_loop_start_tick: u64,
    pub cycle_loop_end_tick: u64,
    pub cycle_loop_enabled: bool,
}

impl Default for CrystalWaveProject {
    fn default() -> Self {
        Self {
            metadata: ProjectMetadata::default(),
            tracks: Vec::new(),
            cycle_loop_start_tick: 0,
            cycle_loop_end_tick: 3840 * 4,
            cycle_loop_enabled: false,
        }
    }
}

impl CrystalWaveProject {
    pub fn new(name: impl Into<String>, sample_rate: f32, bpm: f32) -> Self {
        Self {
            metadata: ProjectMetadata {
                name: name.into(),
                author: "Producer".into(),
                sample_rate,
                bpm,
                time_signature: (4, 4),
                format_version: "12.3.1".into(),
            },
            tracks: Vec::new(),
            cycle_loop_start_tick: 0,
            cycle_loop_end_tick: 3840 * 4,
            cycle_loop_enabled: false,
        }
    }

    pub fn add_track(&mut self, track: SerializedTrack) {
        self.tracks.push(track);
    }
}
