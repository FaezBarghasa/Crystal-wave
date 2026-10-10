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

#[derive(Debug, Clone, PartialEq)]
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
