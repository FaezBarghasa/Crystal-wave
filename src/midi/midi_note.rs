#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ArticulationType {
    #[default]
    Attribute, // Applies strictly to this specific note
    Direction, // Persists across subsequent notes until changed
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArticulationMapEntry {
    pub articulation_id: u8, // 1..255
    pub name: String,
    pub trigger_note: Option<u8>,        // Keyswitch note trigger
    pub program_change: Option<u8>,
    pub control_change: Option<(u8, u8)>, // (CC#, Value)
    pub articulation_type: ArticulationType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MpeExpression {
    pub pitch_bend_semitones: f32, // +/- 96 semitones per note
    pub pressure: u16,             // 14-bit poly pressure / aftertouch (0..16383)
    pub timbre: u16,               // 14-bit brightness CC 74 (0..16383)
}

impl Default for MpeExpression {
    fn default() -> Self {
        Self {
            pitch_bend_semitones: 0.0,
            pressure: 0,
            timbre: 8192, // Centered
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MidiNote {
    pub id: usize,
    pub start_tick: u64,
    pub duration_ticks: u32,
    pub pitch: u8,               // 0..127 MIDI note number
    pub velocity: u16,           // 16-bit high-resolution velocity (MIDI 2.0, 0..65535)
    pub release_velocity: u16,
    pub articulation_id: u8,     // Logic Pro Articulation ID (1..255, 0 = default)
    pub mpe: MpeExpression,
    pub channel: u8,             // 1..16
    pub muted: bool,
}

impl MidiNote {
    pub fn new(id: usize, start_tick: u64, duration_ticks: u32, pitch: u8, velocity_7bit: u8) -> Self {
        Self {
            id,
            start_tick,
            duration_ticks,
            pitch: pitch.clamp(0, 127),
            velocity: (velocity_7bit as u16) << 9, // Upscale 7-bit to 16-bit
            release_velocity: 64 << 9,
            articulation_id: 0,
            mpe: MpeExpression::default(),
            channel: 1,
            muted: false,
        }
    }

    pub fn with_mpe(mut self, pitch_bend: f32, pressure: u16, timbre: u16) -> Self {
        self.mpe = MpeExpression {
            pitch_bend_semitones: pitch_bend.clamp(-96.0, 96.0),
            pressure: pressure.min(16383),
            timbre: timbre.min(16383),
        };
        self
    }

    pub fn with_articulation(mut self, articulation_id: u8) -> Self {
        self.articulation_id = articulation_id;
        self
    }

    /// Color code representing note velocity (blue -> green -> yellow -> red)
    pub fn velocity_color_rgb(&self) -> (u8, u8, u8) {
        let norm = (self.velocity as f32 / 65535.0).clamp(0.0, 1.0);
        if norm < 0.33 {
            let t = norm / 0.33;
            // Blue to Cyan/Green
            (0, (t * 255.0) as u8, 255)
        } else if norm < 0.66 {
            let t = (norm - 0.33) / 0.33;
            // Green to Yellow
            ((t * 255.0) as u8, 255, (255.0 * (1.0 - t)) as u8)
        } else {
            let t = (norm - 0.66) / 0.34;
            // Yellow to Red
            (255, (255.0 * (1.0 - t)) as u8, 0)
        }
    }
}
