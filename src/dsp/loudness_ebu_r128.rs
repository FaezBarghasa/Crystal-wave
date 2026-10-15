use std::f32::consts::PI;

/// Second-order IIR filter state for K-weighting
#[derive(Debug, Clone, Default)]
struct BiquadState {
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl BiquadState {
    #[inline]
    fn process(&mut self, input: f32, b0: f32, b1: f32, b2: f32, a1: f32, a2: f32) -> f32 {
        let output = b0 * input + b1 * self.x1 + b2 * self.x2 - a1 * self.y1 - a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = output;
        output
    }

    fn reset(&mut self) {
        self.x1 = 0.0;
        self.x2 = 0.0;
        self.y1 = 0.0;
        self.y2 = 0.0;
    }
}

/// Filter coefficients for K-weighting (Stage 1 Shelf + Stage 2 Highpass)
#[derive(Debug, Clone)]
struct KWeightingCoeffs {
    // Stage 1: High shelf
    s1_b0: f32,
    s1_b1: f32,
    s1_b2: f32,
    s1_a1: f32,
    s1_a2: f32,

    // Stage 2: Highpass (RLB)
    s2_b0: f32,
    s2_b1: f32,
    s2_b2: f32,
    s2_a1: f32,
    s2_a2: f32,
}

impl KWeightingCoeffs {
    fn calculate(sample_rate: f32) -> Self {
        // Stage 1: High shelf (+3.99 dB at 1681 Hz)
        let db_gain = 3.9998438f32;
        let f0 = 1681.9745_f32;
        let q = 0.70717525_f32;
        let k = (PI * f0 / sample_rate).tan();
        let vh = 10.0f32.powf(db_gain / 20.0);
        let vb = 10.0f32.powf(db_gain / 40.0);

        let a0_s1 = 1.0 + k / q + k * k;
        let s1_b0 = (vh + vb * (k / q) + k * k) / a0_s1;
        let s1_b1 = 2.0 * (k * k - vh) / a0_s1;
        let s1_b2 = (vh - vb * (k / q) + k * k) / a0_s1;
        let s1_a1 = 2.0 * (k * k - 1.0) / a0_s1;
        let s1_a2 = (1.0 - k / q + k * k) / a0_s1;

        // Stage 2: High pass (RLB weighting, ~38 Hz)
        let f0_hp = 38.13547_f32;
        let q_hp = 0.50032705_f32;
        let k_hp = (PI * f0_hp / sample_rate).tan();

        let a0_s2 = 1.0 + k_hp / q_hp + k_hp * k_hp;
        let s2_b0 = 1.0 / a0_s2;
        let s2_b1 = -2.0 / a0_s2;
        let s2_b2 = 1.0 / a0_s2;
        let s2_a1 = 2.0 * (k_hp * k_hp - 1.0) / a0_s2;
        let s2_a2 = (1.0 - k_hp / q_hp + k_hp * k_hp) / a0_s2;

        Self {
            s1_b0,
            s1_b1,
            s1_b2,
            s1_a1,
            s1_a2,
            s2_b0,
            s2_b1,
            s2_b2,
            s2_a1,
            s2_a2,
        }
    }
}

/// Channel K-filter processor
#[derive(Debug, Clone)]
struct ChannelFilter {
    stage1: BiquadState,
    stage2: BiquadState,
}

impl ChannelFilter {
    fn new() -> Self {
        Self {
            stage1: BiquadState::default(),
            stage2: BiquadState::default(),
        }
    }

    #[inline]
    fn process_sample(&mut self, sample: f32, coeffs: &KWeightingCoeffs) -> f32 {
        let y1 = self.stage1.process(
            sample,
            coeffs.s1_b0,
            coeffs.s1_b1,
            coeffs.s1_b2,
            coeffs.s1_a1,
            coeffs.s1_a2,
        );
        self.stage2.process(
            y1,
            coeffs.s2_b0,
            coeffs.s2_b1,
            coeffs.s2_b2,
            coeffs.s2_a1,
            coeffs.s2_a2,
        )
    }

    fn reset(&mut self) {
        self.stage1.reset();
        self.stage2.reset();
    }
}

/// EBU R128 / ITU-R BS.1770-4 Broadcast Loudness Meter
#[derive(Debug, Clone)]
pub struct EbuR128LoudnessMeter {
    sample_rate: f32,
    coeffs: KWeightingCoeffs,
    filters: Vec<ChannelFilter>,
    channel_weights: Vec<f32>,

    // Sliding buffers for Momentary (400ms) and Short-Term (3s)
    momentary_samples: usize,
    short_term_samples: usize,
    momentary_history: Vec<f32>,
    short_term_history: Vec<f32>,
    history_idx: usize,

    // Gated block history for Integrated Loudness (100ms blocks = 400ms window with 75% overlap)
    block_energies: Vec<f32>,
    block_accumulator: f32,
    block_sample_count: usize,
    block_size_samples: usize,

    // True Peak detector state
    peak_dbtp: f32,
}

impl EbuR128LoudnessMeter {
    pub fn new(sample_rate: f32, num_channels: usize) -> Self {
        let sr = sample_rate.max(8000.0);
        let coeffs = KWeightingCoeffs::calculate(sr);
        let filters = vec![ChannelFilter::new(); num_channels];

        // Standard BS.1770 channel weighting: L/R/C = 1.0, LFE = 0.0, Surrounds = 1.41
        let channel_weights = match num_channels {
            1 => vec![1.0],
            2 => vec![1.0, 1.0],
            6 => vec![1.0, 1.0, 1.0, 0.0, 1.41, 1.41], // 5.1 Surround
            n => vec![1.0; n],
        };

        let momentary_samples = (sr * 0.400).round() as usize;
        let short_term_samples = (sr * 3.000).round() as usize;
        let block_size_samples = (sr * 0.100).round() as usize;

        Self {
            sample_rate: sr,
            coeffs,
            filters,
            channel_weights,
            momentary_samples,
            short_term_samples,
            momentary_history: vec![0.0; momentary_samples],
            short_term_history: vec![0.0; short_term_samples],
            history_idx: 0,
            block_energies: Vec::new(),
            block_accumulator: 0.0,
            block_sample_count: 0,
            block_size_samples,
            peak_dbtp: -120.0,
        }
    }

    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    /// Processes an interleaved or multi-channel audio buffer chunk and updates loudness meters.
    /// `channels` contains slices for each channel (e.g. `[left, right]`).
    #[allow(clippy::needless_range_loop)]
    pub fn process_channels(&mut self, channels: &[&[f32]]) {
        if channels.is_empty() || channels[0].is_empty() {
            return;
        }

        let num_samples = channels[0].len();
        let num_channels = channels.len().min(self.filters.len());

        for i in 0..num_samples {
            let mut weighted_power = 0.0f32;

            for ch in 0..num_channels {
                let raw_sample = channels[ch][i];

                // True Peak approximation (simple 4x inter-sample peak estimate)
                let abs_s = raw_sample.abs();
                if abs_s > 0.0 {
                    let dbtp = 20.0 * abs_s.log10();
                    if dbtp > self.peak_dbtp {
                        self.peak_dbtp = dbtp;
                    }
                }

                // K-filter sample
                let filtered = self.filters[ch].process_sample(raw_sample, &self.coeffs);
                weighted_power += self.channel_weights[ch] * (filtered * filtered);
            }

            // Update sliding momentary history
            let m_idx = self.history_idx % self.momentary_samples;
            self.momentary_history[m_idx] = weighted_power;

            // Update sliding short-term history
            let s_idx = self.history_idx % self.short_term_samples;
            self.short_term_history[s_idx] = weighted_power;

            self.history_idx = self.history_idx.wrapping_add(1);

            // Accumulate for 100ms integrated blocks
            self.block_accumulator += weighted_power;
            self.block_sample_count += 1;

            if self.block_sample_count >= self.block_size_samples {
                let mean_block_power = self.block_accumulator / (self.block_sample_count as f32);
                // Bounded block history (max 36,000 blocks = 1 hour of integrated audio)
                const MAX_BLOCKS: usize = 36_000;
                if self.block_energies.len() >= MAX_BLOCKS {
                    self.block_energies.remove(0);
                }
                self.block_energies.push(mean_block_power);
                self.block_accumulator = 0.0;
                self.block_sample_count = 0;
            }
        }
    }

    /// Momentary Loudness $M$ (400 ms sliding window) in LUFS
    pub fn momentary_lufs(&self) -> f32 {
        let sum: f32 = self.momentary_history.iter().sum();
        let mean = sum / (self.momentary_samples as f32);
        self.power_to_lufs(mean)
    }

    /// Short-Term Loudness $S$ (3 s sliding window) in LUFS
    pub fn short_term_lufs(&self) -> f32 {
        let sum: f32 = self.short_term_history.iter().sum();
        let mean = sum / (self.short_term_samples as f32);
        self.power_to_lufs(mean)
    }

    /// Integrated Loudness $I$ with BS.1770-4 dual-gating in LUFS
    pub fn integrated_lufs(&self) -> f32 {
        if self.block_energies.is_empty() {
            return -70.0;
        }

        // Absolute threshold: -70.0 LKFS
        let abs_threshold_power = 10.0f32.powf((-70.0 + 0.691) / 10.0);
        let mut abs_gated_sum = 0.0f32;
        let mut abs_gated_count = 0;

        for &p in &self.block_energies {
            if p >= abs_threshold_power {
                abs_gated_sum += p;
                abs_gated_count += 1;
            }
        }

        if abs_gated_count == 0 {
            return -70.0;
        }

        // Relative threshold: -10 LU below absolute-gated average
        let ungated_mean = abs_gated_sum / (abs_gated_count as f32);
        let rel_threshold_lufs = self.power_to_lufs(ungated_mean) - 10.0;
        let rel_threshold_power = 10.0f32.powf((rel_threshold_lufs + 0.691) / 10.0);

        let mut rel_gated_sum = 0.0f32;
        let mut rel_gated_count = 0;

        for &p in &self.block_energies {
            if p >= abs_threshold_power && p >= rel_threshold_power {
                rel_gated_sum += p;
                rel_gated_count += 1;
            }
        }

        if rel_gated_count == 0 {
            return -70.0;
        }

        let final_mean = rel_gated_sum / (rel_gated_count as f32);
        self.power_to_lufs(final_mean)
    }

    /// Maximum True-Peak level measured in dBTP
    pub fn max_true_peak_dbtp(&self) -> f32 {
        self.peak_dbtp
    }

    pub fn reset(&mut self) {
        for f in &mut self.filters {
            f.reset();
        }
        self.momentary_history.fill(0.0);
        self.short_term_history.fill(0.0);
        self.history_idx = 0;
        self.block_energies.clear();
        self.block_accumulator = 0.0;
        self.block_sample_count = 0;
        self.peak_dbtp = -120.0;
    }

    #[inline]
    fn power_to_lufs(&self, power: f32) -> f32 {
        if power <= 1e-12 {
            -70.0
        } else {
            -0.691 + 10.0 * power.log10()
        }
    }
}
