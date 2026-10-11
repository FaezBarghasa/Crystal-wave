#[derive(Debug, Clone, PartialEq)]
pub struct ReturnNode {
    pub bus_id: usize,
    pub volume_fader_db: f32,
    pub muted: bool,
    pub solo: bool,
    pub phase_invert: bool,
}

impl ReturnNode {
    pub fn new(bus_id: usize) -> Self {
        Self {
            bus_id,
            volume_fader_db: 0.0,
            muted: false,
            solo: false,
            phase_invert: false,
        }
    }

    pub fn linear_gain(&self) -> f32 {
        if self.muted || self.volume_fader_db <= -96.0 {
            0.0
        } else {
            10.0f32.powf(self.volume_fader_db / 20.0)
        }
    }
}
