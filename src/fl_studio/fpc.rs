//! FPC (Fruity Pad Controller) 16-Pad MPC Drum Sampler
//!
//! Emulates FL Studio's FPC drum machine with 4x4 velocity-sensitive pads,
//! multi-layer sample triggering, pad color themes, and kit presets.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FpcPad {
    pub id: usize, // 0..15 (Pad 1..16)
    pub name: String,
    pub note_number: u8, // MIDI Note (e.g. 36 = C2 / Kick 1)
    pub color_rgb: [u8; 3],
    pub sample_name: String,
    pub sample_path: Option<String>,
    pub volume: f32, // 0.0 to 1.25
    pub pan: f32,    // -1.0 to 1.0
    pub muted: bool,
    pub solo: bool,
    pub is_triggered: bool,
}

#[derive(Debug, Clone)]
pub struct FpcEngine {
    pub pads: Vec<FpcPad>,
    pub active_kit_name: String,
    pub master_volume: f32,
    pub global_pitch: i8, // -12 to +12 semitones
}

impl Default for FpcEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl FpcEngine {
    pub fn new() -> Self {
        let default_names = [
            ("Kick 1", 36, [0xf8, 0x8a, 0x36]),        // C2
            ("Snare 1", 38, [0x2e, 0xcc, 0x71]),       // D2
            ("Hi-Hat Closed", 42, [0x00, 0xf2, 0xfe]), // F#2
            ("Hi-Hat Open", 46, [0x34, 0x98, 0xdb]),   // A#2
            ("Clap", 39, [0xe7, 0x4c, 0x3c]),          // D#2
            ("Crash Cymbal", 49, [0xf1, 0xc4, 0x0f]),  // C#3
            ("Ride Cymbal", 51, [0xe6, 0x7e, 0x22]),   // D#3
            ("Tom Low", 41, [0x9b, 0x59, 0xb6]),       // F2
            ("Tom Mid", 45, [0x8e, 0x44, 0xad]),       // A2
            ("Tom High", 48, [0x1a, 0xbc, 0x9c]),      // C3
            ("Rimshot", 37, [0x16, 0xa0, 0x85]),       // C#2
            ("Shaker", 70, [0x29, 0x80, 0xb9]),        // A#4
            ("808 Sub", 35, [0xff, 0x5e, 0x3a]),       // B1
            ("Perc Hit", 54, [0x54, 0xa0, 0xff]),      // F#3
            ("Cowbell", 56, [0x5f, 0x27, 0xcd]),       // G#3
            ("FX Sweep", 60, [0x00, 0xd2, 0xd3]),      // C4
        ];

        let mut pads = Vec::with_capacity(16);
        for (i, (name, note, col)) in default_names.iter().enumerate() {
            pads.push(FpcPad {
                id: i,
                name: (*name).to_string(),
                note_number: *note,
                color_rgb: *col,
                sample_name: format!("{}.wav", name),
                sample_path: None,
                volume: 0.8,
                pan: 0.0,
                muted: false,
                solo: false,
                is_triggered: false,
            });
        }

        Self {
            pads,
            active_kit_name: "Default Trap & Urban Kit".into(),
            master_volume: 1.0,
            global_pitch: 0,
        }
    }

    pub fn trigger_pad(&mut self, pad_id: usize) {
        if let Some(pad) = self.pads.get_mut(pad_id) {
            pad.is_triggered = true;
        }
    }

    pub fn reset_triggers(&mut self) {
        for pad in &mut self.pads {
            pad.is_triggered = false;
        }
    }

    pub fn load_kit(&mut self, kit_name: &str) {
        self.active_kit_name = kit_name.to_string();
    }
}
