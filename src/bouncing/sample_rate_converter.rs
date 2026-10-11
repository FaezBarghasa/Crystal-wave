use std::f32::consts::PI;

pub struct SampleRateConverter {
    pub source_rate: f32,
    pub target_rate: f32,
}

impl SampleRateConverter {
    pub fn new(source_rate: f32, target_rate: f32) -> Self {
        Self {
            source_rate,
            target_rate,
        }
    }

    /// Sinc function: sin(pi * x) / (pi * x)
    fn sinc(x: f32) -> f32 {
        if x.abs() < 1e-6 {
            1.0
        } else {
            let px = PI * x;
            px.sin() / px
        }
    }

    /// Resample buffer using windowed polyphase sinc interpolation
    pub fn resample(&self, input: &[f32]) -> Vec<f32> {
        if input.is_empty() {
            return Vec::new();
        }
        if (self.source_rate - self.target_rate).abs() < 1e-3 {
            return input.to_vec();
        }

        let ratio = self.target_rate / self.source_rate;
        let out_len = ((input.len() as f32) * ratio).round() as usize;
        let mut output = Vec::with_capacity(out_len);

        let filter_radius = 8usize; // 16-tap polyphase sinc window
        let cutoff = if ratio < 1.0 { ratio } else { 1.0 }; // Anti-aliasing bandwidth clamp

        for out_idx in 0..out_len {
            let in_pos = out_idx as f32 / ratio;
            let center = in_pos.floor() as isize;

            let mut sum = 0.0f32;
            let mut weight_sum = 0.0f32;

            for tap in (center - filter_radius as isize)..=(center + filter_radius as isize) {
                if tap >= 0 && (tap as usize) < input.len() {
                    let diff = in_pos - (tap as f32);
                    // Blackman window
                    let w_arg =
                        2.0 * PI * (diff + filter_radius as f32) / (2.0 * filter_radius as f32);
                    let window = 0.42 - 0.5 * w_arg.cos() + 0.08 * (2.0 * w_arg).cos();
                    let weight = Self::sinc(diff * cutoff) * window;

                    sum += input[tap as usize] * weight;
                    weight_sum += weight;
                }
            }

            if weight_sum.abs() > 1e-6 {
                output.push(sum / weight_sum);
            } else {
                output.push(0.0);
            }
        }

        output
    }
}
