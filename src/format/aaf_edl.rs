use crate::format::timecode::{SmpteFrameRate, SmpteTimecode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdlTransition {
    Cut,
    Dissolve,
    Wipe,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EdlEvent {
    pub event_num: usize,
    pub reel_name: String,
    pub track_type: String, // "V", "A", "A2"
    pub transition: EdlTransition,
    pub src_in: SmpteTimecode,
    pub src_out: SmpteTimecode,
    pub master_in: SmpteTimecode,
    pub master_out: SmpteTimecode,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AafClip {
    pub name: String,
    pub start_timecode: SmpteTimecode,
    pub length_frames: u64,
    pub gain_db: f32,
    pub pan: f32,
    pub fade_in_frames: u32,
    pub fade_out_frames: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AafTrack {
    pub name: String,
    pub is_audio: bool,
    pub clips: Vec<AafClip>,
}

/// Universal Session Interchange representation for AAF, OMF, and EDL files
#[derive(Debug, Clone, PartialEq)]
pub struct AafSession {
    pub project_name: String,
    pub sample_rate: f32,
    pub frame_rate: SmpteFrameRate,
    pub tracks: Vec<AafTrack>,
    pub edl_events: Vec<EdlEvent>,
}

impl AafSession {
    pub fn new(name: &str, sample_rate: f32, frame_rate: SmpteFrameRate) -> Self {
        Self {
            project_name: name.to_string(),
            sample_rate,
            frame_rate,
            tracks: Vec::new(),
            edl_events: Vec::new(),
        }
    }

    pub fn add_track(&mut self, name: &str, is_audio: bool) -> usize {
        let idx = self.tracks.len();
        self.tracks.push(AafTrack {
            name: name.to_string(),
            is_audio,
            clips: Vec::new(),
        });
        idx
    }

    /// Serializes session events into standard CMX 3600 EDL string
    pub fn export_cmx3600_edl(&self) -> String {
        let mut out = format!("TITLE: {}\nFCM: NON-DROP FRAME\n\n", self.project_name);
        for ev in &self.edl_events {
            let tr_str = match ev.transition {
                EdlTransition::Cut => "C",
                EdlTransition::Dissolve => "D",
                EdlTransition::Wipe => "W",
            };
            out.push_str(&format!(
                "{:03}  {:8} {}     {}        {} {} {} {}\n",
                ev.event_num,
                ev.reel_name,
                ev.track_type,
                tr_str,
                ev.src_in.to_string_formatted(),
                ev.src_out.to_string_formatted(),
                ev.master_in.to_string_formatted(),
                ev.master_out.to_string_formatted()
            ));
        }
        out
    }
}
