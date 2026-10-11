/// Fruity Soft Clipper / Limiter for punchy, saturated 808s and aggressive clipping
#[derive(Debug, Clone)]
pub struct FruitySoftClipper {
    pub threshold_db: f32, // -12.0 to 0.0 dB
    pub post_gain_db: f32, // 0.0 to +12.0 dB
    pub soft_knee: f32,    // 0.0 (hard) to 1.0 (smooth tanh)
}

impl Default for FruitySoftClipper {
    fn default() -> Self {
        Self {
            threshold_db: -0.1,
            post_gain_db: 0.0,
            soft_knee: 0.6,
        }
    }
}

impl FruitySoftClipper {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn process_sample(&self, input: f32) -> f32 {
        let thresh_lin = 10.0f32.powf(self.threshold_db / 20.0);
        let abs_in = input.abs();

        let clipped = if abs_in <= thresh_lin {
            input
        } else {
            let sign = input.signum();
            let over = abs_in - thresh_lin;
            let knee = self.soft_knee.max(0.01);
            let compressed_over = (over / knee).tanh() * knee;
            sign * (thresh_lin + compressed_over)
        };

        let post_gain = 10.0f32.powf(self.post_gain_db / 20.0);
        clipped * post_gain
    }

    pub fn process(&self, left: &mut [f32], right: &mut [f32]) {
        for s in left.iter_mut() {
            *s = self.process_sample(*s);
        }
        for s in right.iter_mut() {
            *s = self.process_sample(*s);
        }
    }
}
