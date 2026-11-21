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
            // Row 0 (bottom row, pads 0..3): SideStick, Kick Drum, Closed Hat, Tambourine
            ("SideStick", 37, [0x3e, 0x8f, 0x9f]), // Cyan / Teal
            ("Kick Drum", 36, [0xbd, 0x51, 0x51]), // Coral / Warm Red
            ("Closed Hat", 42, [0x59, 0xa5, 0x5a]), // Warm Green
            ("Tambourine", 54, [0xb8, 0x8a, 0x42]), // Gold / Brown
            // Row 1 (pads 4..7): Snare 1, Snare 2, Open HiHat, Pedal Hi Hat
            ("Snare 1", 38, [0x46, 0x9b, 0xaa]), // Teal / Cyan
            ("Snare 2", 40, [0x46, 0x9b, 0xaa]), // Teal / Cyan
            ("Open HiHat", 46, [0x59, 0xa5, 0x5a]), // Warm Green
            ("Pedal Hi Hat", 44, [0x59, 0xa5, 0x5a]), // Warm Green
            // Row 2 (pads 8..11): Hi Tom, Mid Tom, Lo Tom, Floor Tom
            ("Hi Tom", 50, [0xb8, 0x8a, 0x42]), // Gold / Ochre
            ("Mid Tom", 47, [0xb8, 0x8a, 0x42]), // Gold / Ochre
            ("Lo Tom", 45, [0xb8, 0x8a, 0x42]), // Gold / Ochre
            ("Floor Tom", 41, [0xb8, 0x8a, 0x42]), // Gold / Ochre
            // Row 3 (top row, pads 12..15): Crash, Lite Crash, Lite Ride, Ride Bell
            ("Crash", 49, [0x68, 0xb0, 0x4d]), // Fresh Green
            ("Lite Crash", 57, [0x68, 0xb0, 0x4d]), // Fresh Green
            ("Lite Ride", 51, [0x48, 0xb8, 0xa5]), // Turquoise / Sea Green
            ("Ride Bell", 53, [0x48, 0xb8, 0xa5]), // Turquoise / Sea Green
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
