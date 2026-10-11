#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LevelMeter {
    pub peak_db: f32,
    pub true_peak_db: f32,
    pub rms_db: f32,
    pub peak_hold_db: f32,
    pub gain_reduction_db: f32,
    pub auto_mute_tripped: bool,
}

impl Default for LevelMeter {
    fn default() -> Self {
        Self {
            peak_db: -96.0,
            true_peak_db: -96.0,
            rms_db: -96.0,
            peak_hold_db: -96.0,
            gain_reduction_db: 0.0,
            auto_mute_tripped: false,
        }
    }
}

impl LevelMeter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Process audio block, compute True Peak, RMS, and monitor for +12 dBFS overload
    pub fn process_block(&mut self, left: &mut [f32], right: &mut [f32]) {
        let mut max_abs = 0.0f32;
        let mut sum_sq = 0.0f32;
        let len = left.len().min(right.len());

        if len == 0 {
            return;
        }

        for i in 0..len {
            let l = left[i];
            let r = right[i];
            let abs_val = l.abs().max(r.abs());
            if abs_val > max_abs {
                max_abs = abs_val;
            }
            sum_sq += l * l + r * r;
        }

        // True Peak estimation using 4-sample quadratic inter-sample interpolation
        let mut true_peak_linear = max_abs;
        for i in 1..len.saturating_sub(1) {
            let y0 = left[i - 1].abs();
            let y1 = left[i].abs();
            let y2 = left[i + 1].abs();
            let denom = 2.0 * (2.0 * y1 - y0 - y2);
            if denom.abs() > 1e-6 {
                let peak_interp = y1 + ((y2 - y0).powi(2)) / (8.0 * denom);
                if peak_interp > true_peak_linear {
                    true_peak_linear = peak_interp;
                }
            }
        }

        // +12 dBFS overload threshold: 10^(12/20) ~= 3.981
        if true_peak_linear > 3.981 {
            self.auto_mute_tripped = true;
            for s in left.iter_mut() { *s = 0.0; }
            for s in right.iter_mut() { *s = 0.0; }
            self.peak_db = 12.0;
            self.true_peak_db = 12.0;
            return;
        }

        self.peak_db = if max_abs > 1e-5 { 20.0 * max_abs.log10() } else { -96.0 };
        self.true_peak_db = if true_peak_linear > 1e-5 { 20.0 * true_peak_linear.log10() } else { -96.0 };
        let mean_sq = sum_sq / (len * 2) as f32;
        self.rms_db = if mean_sq > 1e-9 { 10.0 * mean_sq.log10() } else { -96.0 };

        if self.peak_db > self.peak_hold_db {
            self.peak_hold_db = self.peak_db;
        } else {
            // Decay peak hold slowly
            self.peak_hold_db = (self.peak_hold_db - 0.05).max(self.peak_db);
        }
    }

    pub fn reset_auto_mute(&mut self) {
        self.auto_mute_tripped = false;
    }
}
