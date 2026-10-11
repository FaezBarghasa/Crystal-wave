use crate::timeline::marker::Marker;
use crate::timeline::region::Region;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackType {
    Audio,
    Software,
    Drum,
    Midi,
    Aux,
    Master,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FreezeMode {
    #[default]
    None,
    SourceFreeze,
    PreFaderFreeze,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VarispeedMode {
    #[default]
    SpeedOnly,
    VarispeedAndPitch,
    MidiOnly,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VarispeedUnits {
    Percent(f32),      // -50.0 to +50.0%
    Semitones(f32),    // -7.0 to +7.0 semitones
    BpmOffset(f32),    // exact BPM difference
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrackAlternative {
    pub id: usize,
    pub name: String,
    pub regions: Vec<Region>,
}

impl TrackAlternative {
    pub fn new(id: usize, name: impl Into<String>, regions: Vec<Region>) -> Self {
        Self {
            id,
            name: name.into(),
            regions,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SendNode {
    pub bus_id: usize,
    pub gain_db: f32,
    pub pre_fader: bool,
    pub muted: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReturnNode {
    pub bus_id: usize,
    pub gain_db: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Default for Color {
    fn default() -> Self {
        Self { r: 60, g: 120, b: 200 }
    }
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Calculate relative luminance (ITU-R BT.709)
    pub fn luminance(&self) -> f32 {
        0.2126 * (self.r as f32 / 255.0)
            + 0.7152 * (self.g as f32 / 255.0)
            + 0.0722 * (self.b as f32 / 255.0)
    }

    /// Automatic text contrast inversion
    pub fn text_color(&self) -> Self {
        if self.luminance() > 0.5 {
            Self::new(20, 20, 20)
        } else {
            Self::new(240, 240, 240)
        }
    }

    /// 96 curated high-contrast hues for Logic Pro palette
    pub fn curated_palette(index: usize) -> Self {
        let palette_idx = index % 96;
        let hue = (palette_idx as f32 * (360.0 / 96.0)) % 360.0;
        let s = 0.75f32;
        let v = 0.85f32;
        let c = v * s;
        let x = c * (1.0 - ((hue / 60.0) % 2.0 - 1.0).abs());
        let m = v - c;

        let (r1, g1, b1) = if hue < 60.0 {
            (c, x, 0.0)
        } else if hue < 120.0 {
            (x, c, 0.0)
        } else if hue < 180.0 {
            (0.0, c, x)
        } else if hue < 240.0 {
            (0.0, x, c)
        } else if hue < 300.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };

        Self::new(
            ((r1 + m) * 255.0).round() as u8,
            ((g1 + m) * 255.0).round() as u8,
            ((b1 + m) * 255.0).round() as u8,
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Track {
    pub id: usize,
    pub name: String,
    pub track_type: TrackType,
    pub regions: Vec<Region>,
    pub sends: Vec<SendNode>,
    pub returns: Vec<ReturnNode>,
    pub color: Color,
    pub markers: Vec<Marker>,
    pub bypassed: bool,
    pub muted: bool,
    pub soloed: bool,
    pub record_armed: bool,
    pub hidden: bool,
    pub freeze_mode: FreezeMode,
    pub is_frozen: bool,
    pub frozen_audio_buffer: Option<Vec<f32>>,
    pub alternatives: Vec<TrackAlternative>,
    pub active_alternative_index: usize,
    pub low_latency_safe: bool,
    pub delay_offset_ticks: i64,
    pub delay_offset_ms: f32,
    pub phase_invert: bool,
    pub gain_db: f32,
    pub pan: f32, // -1.0 to 1.0
}

impl Track {
    pub fn new(id: usize, name: impl Into<String>, track_type: TrackType) -> Self {
        Self {
            id,
            name: name.into(),
            track_type,
            regions: Vec::new(),
            sends: Vec::new(),
            returns: Vec::new(),
            color: Color::default(),
            markers: Vec::new(),
            bypassed: false,
            muted: false,
            soloed: false,
            record_armed: false,
            hidden: false,
            freeze_mode: FreezeMode::None,
            is_frozen: false,
            frozen_audio_buffer: None,
            alternatives: Vec::new(),
            active_alternative_index: 0,
            low_latency_safe: true,
            delay_offset_ticks: 0,
            delay_offset_ms: 0.0,
            phase_invert: false,
            gain_db: 0.0,
            pan: 0.0,
        }
    }

    pub fn solo(&mut self, solo: bool) {
        self.soloed = solo;
    }

    pub fn mute(&mut self, mute: bool) {
        self.muted = mute;
    }

    pub fn arm(&mut self, arm: bool) {
        self.record_armed = arm;
    }

    pub fn bypass(&mut self, bypass: bool) {
        self.bypassed = bypass;
    }

    pub fn set_hidden(&mut self, hidden: bool) {
        self.hidden = hidden;
    }

    pub fn freeze(&mut self, mode: FreezeMode, rendered_audio: Option<Vec<f32>>) {
        self.freeze_mode = mode;
        self.is_frozen = mode != FreezeMode::None;
        self.frozen_audio_buffer = rendered_audio;
    }

    pub fn unfreeze(&mut self) {
        self.freeze_mode = FreezeMode::None;
        self.is_frozen = false;
        self.frozen_audio_buffer = None;
    }

    pub fn add_alternative(&mut self, name: impl Into<String>) -> usize {
        let alt_id = self.alternatives.len();
        self.alternatives.push(TrackAlternative::new(
            alt_id,
            name,
            self.regions.clone(),
        ));
        alt_id
    }

    pub fn duplicate_alternative(&mut self, name: impl Into<String>) -> usize {
        let new_id = self.alternatives.len();
        self.alternatives.push(TrackAlternative::new(
            new_id,
            name,
            self.regions.clone(),
        ));
        self.active_alternative_index = new_id;
        new_id
    }

    pub fn new_empty_alternative(&mut self, name: impl Into<String>) -> usize {
        // First ensure current is recorded
        if self.alternatives.is_empty() {
            self.alternatives.push(TrackAlternative::new(0, "Alternative A", self.regions.clone()));
        } else if self.active_alternative_index < self.alternatives.len() {
            self.alternatives[self.active_alternative_index].regions = self.regions.clone();
        }
        let new_id = self.alternatives.len();
        self.alternatives.push(TrackAlternative::new(new_id, name, Vec::new()));
        self.regions.clear();
        self.active_alternative_index = new_id;
        new_id
    }

    pub fn switch_alternative(&mut self, alt_index: usize) -> bool {
        if alt_index < self.alternatives.len() {
            if alt_index == self.active_alternative_index {
                return true;
            }
            if self.active_alternative_index < self.alternatives.len() {
                self.alternatives[self.active_alternative_index].regions = self.regions.clone();
            }
            self.regions = self.alternatives[alt_index].regions.clone();
            self.active_alternative_index = alt_index;
            true
        } else {
            false
        }
    }

    pub fn set_gain(&mut self, gain_db: f32) {
        self.gain_db = gain_db;
    }

    pub fn set_pan(&mut self, pan: f32) {
        self.pan = pan.clamp(-1.0, 1.0);
    }

    pub fn stop(&mut self) {
        self.record_armed = false;
    }

    pub fn add_region(&mut self, region: Region) {
        self.regions.push(region);
    }
}
