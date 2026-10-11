#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SendPosition {
    PreFader,
    #[default]
    PostFader,
    PostPan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SendNode {
    pub slot_id: usize,
    pub destination_bus: usize,
    pub level_db: f32, // -inf to +6 dB
    pub position: SendPosition,
    pub muted: bool,
    pub pan: f32, // -1.0 to +1.0
}

impl SendNode {
    pub fn new(slot_id: usize, destination_bus: usize) -> Self {
        Self {
            slot_id,
            destination_bus,
            level_db: 0.0,
            position: SendPosition::PostFader,
            muted: false,
            pan: 0.0,
        }
    }

    pub fn linear_gain(&self) -> f32 {
        if self.muted || self.level_db <= -96.0 {
            0.0
        } else {
            10.0f32.powf(self.level_db / 20.0)
        }
    }
}
