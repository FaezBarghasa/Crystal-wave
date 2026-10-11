pub const MAX_BUSES: usize = 256;

#[derive(Debug, Clone, PartialEq)]
pub struct AuxBus {
    pub id: usize,
    pub name: String,
    pub volume_fader_db: f32,
    pub pan: f32,
    pub muted: bool,
    pub solo: bool,
    pub is_summing_stack: bool,
    pub left_accumulator: Vec<f32>,
    pub right_accumulator: Vec<f32>,
}

impl AuxBus {
    pub fn new(id: usize, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            volume_fader_db: 0.0,
            pan: 0.0,
            muted: false,
            solo: false,
            is_summing_stack: false,
            left_accumulator: Vec::new(),
            right_accumulator: Vec::new(),
        }
    }

    pub fn prepare_buffer(&mut self, block_size: usize) {
        if self.left_accumulator.len() < block_size {
            self.left_accumulator.resize(block_size, 0.0);
            self.right_accumulator.resize(block_size, 0.0);
        }
        self.left_accumulator[..block_size].fill(0.0);
        self.right_accumulator[..block_size].fill(0.0);
    }

    pub fn clear(&mut self) {
        for s in self.left_accumulator.iter_mut() {
            *s = 0.0;
        }
        for s in self.right_accumulator.iter_mut() {
            *s = 0.0;
        }
    }

    pub fn accumulate(&mut self, left: &[f32], right: &[f32], gain: f32) {
        let len = self.left_accumulator.len().min(left.len()).min(right.len());
        for i in 0..len {
            self.left_accumulator[i] += left[i] * gain;
            self.right_accumulator[i] += right[i] * gain;
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
