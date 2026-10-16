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

    /// Construct a fully serialized project snapshot from live DAW state
    pub fn from_session(
        name: &str,
        sample_rate: f32,
        bpm: f32,
        time_sig: (u8, u8),
        tracks: &[crate::timeline::track::Track],
        channel_strips: &[crate::mixing::channel_strip::ChannelStrip],
        loop_range: Option<(u64, u64)>,
    ) -> Self {
        let mut proj = Self::new(name, sample_rate, bpm);
        proj.metadata.time_signature = time_sig;
        if let Some((start, end)) = loop_range {
            proj.cycle_loop_start_tick = start;
            proj.cycle_loop_end_tick = end;
            proj.cycle_loop_enabled = true;
        }

        for (idx, trk) in tracks.iter().enumerate() {
            let ch = channel_strips.get(idx);
            let fader_db = ch.map(|c| c.volume_fader_db).unwrap_or(trk.gain_db);
            let pan = ch.map(|c| c.pan.balance).unwrap_or(trk.pan);
            let muted = ch.map(|c| c.muted).unwrap_or(trk.muted);
            let solo = ch.map(|c| c.solo).unwrap_or(trk.soloed);

            let regions: Vec<SerializedRegion> = trk
                .regions
                .iter()
                .map(|r| SerializedRegion {
                    id: r.id,
                    track_id: r.track_id,
                    name: r.name.clone(),
                    start_tick: r.start_tick,
                    length_ticks: r.length_ticks,
                    muted: r.muted,
                    gain_db: r.gain_db,
                })
                .collect();

            proj.add_track(SerializedTrack {
                id: trk.id,
                name: trk.name.clone(),
                track_type: format!("{:?}", trk.track_type),
                volume_fader_db: fader_db,
                pan,
                muted,
                solo,
                regions,
            });
        }
        proj
    }
}
