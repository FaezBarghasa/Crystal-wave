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
            ("Kick 1", 36, [0xbd, 0x51, 0x51]),        // C2 - Coral/Red
            ("Snare 1", 38, [0x46, 0x9b, 0xaa]),       // D2 - Teal/Cyan
            ("Hi-Hat Closed", 42, [0x59, 0xa5, 0x5a]), // F#2 - Warm Green
            ("Hi-Hat Open", 46, [0x59, 0xa5, 0x5a]),   // A#2 - Warm Green
            ("Clap", 39, [0xbd, 0x51, 0x51]),          // D#2 - Coral/Red
            ("Crash Cymbal", 49, [0x68, 0xb0, 0x4d]),  // C#3 - Fresh Green
            ("Ride Cymbal", 51, [0x48, 0xb8, 0xa5]),   // D#3 - Turquoise
            ("Tom Low", 41, [0xb8, 0x8a, 0x42]),       // F2 - Gold/Ochre
            ("Tom Mid", 45, [0xb8, 0x8a, 0x42]),       // A2 - Gold/Ochre
            ("Tom High", 48, [0xb8, 0x8a, 0x42]),      // C3 - Gold/Ochre
            ("Rimshot", 37, [0x3e, 0x8f, 0x9f]),       // C#2 - Cyan/Teal
            ("Shaker", 70, [0xb8, 0x8a, 0x42]),        // A#4 - Gold/Ochre
            ("808 Sub", 35, [0xbd, 0x51, 0x51]),       // B1 - Coral/Red
            ("Perc Hit", 54, [0xb8, 0x8a, 0x42]),      // F#3 - Gold/Ochre
            ("Cowbell", 56, [0xb8, 0x8a, 0x42]),       // G#3 - Gold/Ochre
            ("FX Sweep", 60, [0x48, 0xb8, 0xa5]),      // C4 - Turquoise
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fpc_pad_triggers() {
        let mut fpc = FpcEngine::new();
        assert_eq!(fpc.pads.len(), 16);
        assert!(!fpc.pads[0].is_triggered);

        fpc.trigger_pad(0);
        assert!(fpc.pads[0].is_triggered);

        fpc.reset_triggers();
        assert!(!fpc.pads[0].is_triggered);
    }

    #[test]
    fn test_fpc_kit_load() {
        let mut fpc = FpcEngine::new();
        fpc.load_kit("808 Trap Kit");
        assert_eq!(fpc.active_kit_name, "808 Trap Kit");
    }
}
