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
    pub target_bus: Option<usize>,
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
            target_bus: None,
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
        let chunks8 = len / 8;
        let gain_vec = wide::f32x8::splat(gain);
        for c in 0..chunks8 {
            let offset = c * 8;
            let l_in = wide::f32x8::new(left[offset..offset + 8].try_into().unwrap_or([0.0; 8]));
            let r_in = wide::f32x8::new(right[offset..offset + 8].try_into().unwrap_or([0.0; 8]));
            let mut l_acc = wide::f32x8::new(
                self.left_accumulator[offset..offset + 8]
                    .try_into()
                    .unwrap_or([0.0; 8]),
            );
            let mut r_acc = wide::f32x8::new(
                self.right_accumulator[offset..offset + 8]
                    .try_into()
                    .unwrap_or([0.0; 8]),
            );
            l_acc += l_in * gain_vec;
            r_acc += r_in * gain_vec;
            self.left_accumulator[offset..offset + 8].copy_from_slice(&l_acc.to_array());
            self.right_accumulator[offset..offset + 8].copy_from_slice(&r_acc.to_array());
        }
        for i in (chunks8 * 8)..len {
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
