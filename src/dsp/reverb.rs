use crate::error::{LogicError, Result};

pub struct ChromaVerbEngine {
    delays_l: [Vec<f32>; 4],
    delays_r: [Vec<f32>; 4],
    indices_l: [usize; 4],
    indices_r: [usize; 4],
    feedback: f32,
    damping: f32,
    damp_state_l: [f32; 4],
    damp_state_r: [f32; 4],
    mix: f32,
}

impl ChromaVerbEngine {
    pub fn new(decay_seconds: f32, sample_rate: f32) -> Result<Self> {
        if sample_rate <= 0.0 {
            return Err(LogicError::InvalidSampleRate(sample_rate));
        }

        // Delay line lengths tuned for prime number sample lengths to prevent metallic ringing
        let base = (sample_rate * 0.030) as usize; // ~30ms base delay
        let lengths = [
            base,
            (base as f32 * 1.37) as usize,
            (base as f32 * 1.63) as usize,
            (base as f32 * 1.91) as usize,
        ];

        let delays_l = [
            vec![0.0; lengths[0]],
            vec![0.0; lengths[1]],
            vec![0.0; lengths[2]],
            vec![0.0; lengths[3]],
        ];

        let delays_r = [
            vec![0.0; lengths[0] + 23],
            vec![0.0; lengths[1] + 37],
            vec![0.0; lengths[2] + 41],
            vec![0.0; lengths[3] + 53],
        ];

        let feedback = (-3.0 / (decay_seconds.max(0.1) * (sample_rate / base as f32)))
            .exp()
            .clamp(0.0, 0.98);

        Ok(Self {
            delays_l,
            delays_r,
            indices_l: [0; 4],
            indices_r: [0; 4],
            feedback,
            damping: 0.3,
            damp_state_l: [0.0; 4],
            damp_state_r: [0.0; 4],
            mix: 0.35,
        })
    }

    pub fn set_mix(&mut self, mix: f32) {
        self.mix = mix.clamp(0.0, 1.0);
    }

    pub fn process(&mut self, left: &mut [f32], right: &mut [f32]) -> Result<()> {
        if left.len() != right.len() {
            return Err(LogicError::BufferLengthMismatch {
                expected: left.len(),
                actual: right.len(),
            });
        }

        for i in 0..left.len() {
            let in_l = left[i];
            let in_r = right[i];

            let mut wet_l = 0.0;
            let mut wet_r = 0.0;

            for d in 0..4 {
                // Read from delay lines
                let len_l = self.delays_l[d].len();
                let len_r = self.delays_r[d].len();

                let out_d_l = self.delays_l[d][self.indices_l[d]];
                let out_d_r = self.delays_r[d][self.indices_r[d]];

                wet_l += out_d_l;
                wet_r += out_d_r;

                // High-frequency damping filter (1-pole lowpass in feedback path)
                self.damp_state_l[d] =
                    out_d_l * (1.0 - self.damping) + self.damp_state_l[d] * self.damping;
                self.damp_state_r[d] =
                    out_d_r * (1.0 - self.damping) + self.damp_state_r[d] * self.damping;

                // Write feedback into delay lines with cross-channel diffusion
                self.delays_l[d][self.indices_l[d]] = in_l
                    + (self.damp_state_l[d] * 0.7 + self.damp_state_r[d] * 0.3) * self.feedback;
                self.delays_r[d][self.indices_r[d]] = in_r
                    + (self.damp_state_r[d] * 0.7 + self.damp_state_l[d] * 0.3) * self.feedback;

                if self.damp_state_l[d].abs() < 1e-15 {
                    self.damp_state_l[d] = 0.0;
                }
                if self.damp_state_r[d].abs() < 1e-15 {
                    self.damp_state_r[d] = 0.0;
                }

                self.indices_l[d] += 1;
                if self.indices_l[d] >= len_l {
                    self.indices_l[d] = 0;
                }
                self.indices_r[d] += 1;
                if self.indices_r[d] >= len_r {
                    self.indices_r[d] = 0;
                }
            }

            wet_l *= 0.25;
            wet_r *= 0.25;

            left[i] = in_l * (1.0 - self.mix) + wet_l * self.mix;
            right[i] = in_r * (1.0 - self.mix) + wet_r * self.mix;
        }

        Ok(())
    }
}
