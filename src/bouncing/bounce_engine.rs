#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BounceMode {
    #[default]
    OfflineHighSpeed,
    RealTime1x,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[derive(Default)]
pub enum NormalizeOption {
    #[default]
    Off,
    PeakNormalize { target_peak_db: f32 },
    LoudnessTargetLUFS { target_lufs: f32 },
    OverloadProtectionOnly, // Attenuates only if clipping occurs (> 0 dBFS)
}


pub struct BounceEngine {
    pub mode: BounceMode,
    pub normalize: NormalizeOption,
    pub second_cycle_pass_enabled: bool,
    pub add_to_audio_bin: bool,
}

impl Default for BounceEngine {
    fn default() -> Self {
        Self {
            mode: BounceMode::OfflineHighSpeed,
            normalize: NormalizeOption::OverloadProtectionOnly,
            second_cycle_pass_enabled: true,
            add_to_audio_bin: false,
        }
    }
}

impl BounceEngine {
    pub fn new(mode: BounceMode, normalize: NormalizeOption) -> Self {
        Self {
            mode,
            normalize,
            second_cycle_pass_enabled: true,
            add_to_audio_bin: false,
        }
    }

    /// Process normalization over stereo buffer
    pub fn apply_normalization(&self, left: &mut [f32], right: &mut [f32]) {
        let max_peak = left
            .iter()
            .chain(right.iter())
            .map(|&s| s.abs())
            .fold(0.0f32, f32::max);

        if max_peak <= 1e-6 {
            return;
        }

        match self.normalize {
            NormalizeOption::Off => {}
            NormalizeOption::OverloadProtectionOnly => {
                if max_peak > 1.0 {
                    let atten = 1.0 / max_peak;
                    for s in left.iter_mut() { *s *= atten; }
                    for s in right.iter_mut() { *s *= atten; }
                }
            }
            NormalizeOption::PeakNormalize { target_peak_db } => {
                let target_linear = 10.0f32.powf(target_peak_db / 20.0);
                let gain = target_linear / max_peak;
                for s in left.iter_mut() { *s *= gain; }
                for s in right.iter_mut() { *s *= gain; }
            }
            NormalizeOption::LoudnessTargetLUFS { target_lufs } => {
                // Integrated RMS/LUFS approximation
                let sum_sq: f32 = left.iter().zip(right.iter()).map(|(&l, &r)| l * l + r * r).sum();
                let rms = (sum_sq / (left.len() * 2) as f32).sqrt();
                if rms > 1e-6 {
                    let current_lufs = 10.0 * rms.log10();
                    let diff_db = target_lufs - current_lufs;
                    let gain = 10.0f32.powf(diff_db / 20.0);
                    for s in left.iter_mut() { *s *= gain; }
                    for s in right.iter_mut() { *s *= gain; }
                }
            }
        }
    }

    /// 2nd Cycle Pass: blends tail of 1st pass into beginning of loop
    pub fn apply_second_cycle_pass(
        &self,
        cycle_left: &mut [f32],
        cycle_right: &mut [f32],
        tail_left: &[f32],
        tail_right: &[f32],
    ) {
        if !self.second_cycle_pass_enabled {
            return;
        }

        let blend_len = cycle_left.len().min(tail_left.len()).min(cycle_right.len()).min(tail_right.len());
        for i in 0..blend_len {
            cycle_left[i] += tail_left[i];
            cycle_right[i] += tail_right[i];
        }
    }
}
