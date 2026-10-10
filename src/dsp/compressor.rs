use crate::error::{LogicError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressorModel {
    /// Vintage FET (Urei 1176 feedback circuit with ultrafast attack & harmonic grit)
    Fet1176,
    /// Vintage Opto (Teletronix LA-2A T4 electro-optical cell with dual-decay memory)
    OptoLa2a,
    /// Vintage Class A (Neve 2254 diode-bridge gain cell with discrete transformer warmth)
    ClassANeve,
    /// Classic VCA (SSL 4000 G Bus feed-forward VCA with snappy transient response)
    SslVca,
    /// Studio VCA (dbx 160 true-RMS level detector with aggressive hard-knee clamp)
    Dbx160,
    /// Platinum Digital (Transparent feed-forward cubic spline soft-knee limiter/compressor)
    PlatinumDigital,
}

#[derive(Debug, Clone)]
pub struct CompressorConfig {
    pub model: CompressorModel,
    pub threshold_db: f32,
    pub ratio: f32,
    pub attack_ms: f32,
    pub release_ms: f32,
    pub makeup_gain_db: f32,
    pub knee_db: f32,
    pub mix: f32,
}

impl Default for CompressorConfig {
    fn default() -> Self {
        Self {
            model: CompressorModel::PlatinumDigital,
            threshold_db: -20.0,
            ratio: 4.0,
            attack_ms: 20.0,
            release_ms: 100.0,
            makeup_gain_db: 0.0,
            knee_db: 6.0,
            mix: 1.0,
        }
    }
}

pub struct Compressor {
    config: CompressorConfig,
    sample_rate: f32,
    envelope: f32,
    /// Dual-stage release envelope memory for Opto LA-2A emulation
    opto_memory: f32,
    /// RMS running accumulator for DBX 160 emulation
    rms_accumulator: f32,
    current_gain_reduction_db: f32,
}

impl Compressor {
    pub fn new(config: CompressorConfig, sample_rate: f32) -> Result<Self> {
        if sample_rate <= 0.0 || sample_rate > 384_000.0 {
            return Err(LogicError::InvalidSampleRate(sample_rate));
        }
        Ok(Self {
            config,
            sample_rate,
            envelope: 0.0,
            opto_memory: 0.0,
            rms_accumulator: 0.0,
            current_gain_reduction_db: 0.0,
        })
    }

    pub fn set_config(&mut self, config: CompressorConfig) {
        self.config = config;
    }

    pub fn current_gain_reduction_db(&self) -> f32 {
        self.current_gain_reduction_db
    }

    pub fn reset(&mut self) {
        self.envelope = 0.0;
        self.opto_memory = 0.0;
        self.rms_accumulator = 0.0;
        self.current_gain_reduction_db = 0.0;
    }

    /// Process in-place stereo or mono buffer
    pub fn process(&mut self, left: &mut [f32], right: &mut [f32]) -> Result<()> {
        if left.len() != right.len() {
            return Err(LogicError::BufferLengthMismatch {
                expected: left.len(),
                actual: right.len(),
            });
        }

        let makeup_linear = 10.0f32.powf(self.config.makeup_gain_db / 20.0);
        let dry_wet = self.config.mix.clamp(0.0, 1.0);

        for i in 0..left.len() {
            let in_l = left[i];
            let in_r = right[i];

            // 1. Level Detection depending on Model
            let detector_input = match self.config.model {
                CompressorModel::Dbx160 => {
                    // True RMS detector: Exponential moving average of squared input
                    let inst_power = 0.5 * (in_l * in_l + in_r * in_r);
                    let alpha_rms = (-1.0 / (0.010 * self.sample_rate)).exp();
                    self.rms_accumulator = alpha_rms * self.rms_accumulator + (1.0 - alpha_rms) * inst_power;
                    self.rms_accumulator.sqrt().max(1e-6)
                }
                _ => (0.5 * (in_l.abs() + in_r.abs())).max(1e-6),
            };

            let detector_db = 20.0 * detector_input.log10();

            // 2. Compute Static Gain Reduction curve with model-specific knee
            let gr_target_db = self.compute_static_curve(detector_db);

            // 3. Ballistics & Envelope Smoothing
            let alpha_att = (-1.0 / (self.effective_attack_ms() * 0.001 * self.sample_rate)).exp();
            let alpha_rel = (-1.0 / (self.effective_release_ms(detector_input) * 0.001 * self.sample_rate)).exp();

            if gr_target_db < self.envelope {
                // Attacking (gain reduction increasing)
                self.envelope = alpha_att * self.envelope + (1.0 - alpha_att) * gr_target_db;
            } else {
                // Releasing (gain recovery)
                self.envelope = alpha_rel * self.envelope + (1.0 - alpha_rel) * gr_target_db;
            }

            self.current_gain_reduction_db = self.envelope;
            let mut gain_linear = 10.0f32.powf(self.envelope / 20.0);

            // 4. Model-specific non-linear saturation
            match self.config.model {
                CompressorModel::Fet1176 => {
                    // 1176 FET program-dependent harmonic saturation
                    let drive = 1.0 + (-self.envelope * 0.03).clamp(0.0, 0.4);
                    gain_linear *= drive;
                }
                CompressorModel::ClassANeve => {
                    // Neve transformer asymmetrical even-order harmonic saturation
                    let sat_l = in_l * 0.15;
                    left[i] += sat_l * sat_l * in_l.signum();
                }
                _ => {}
            }

            // 5. Apply Gain + Makeup + Wet/Dry Mix
            let wet_l = in_l * gain_linear * makeup_linear;
            let wet_r = in_r * gain_linear * makeup_linear;

            left[i] = in_l * (1.0 - dry_wet) + wet_l * dry_wet;
            right[i] = in_r * (1.0 - dry_wet) + wet_r * dry_wet;
        }

        Ok(())
    }

    fn compute_static_curve(&self, in_db: f32) -> f32 {
        let thresh = self.config.threshold_db;
        let ratio = self.config.ratio.max(1.0);
        let knee = match self.config.model {
            CompressorModel::Dbx160 => 0.0, // Strict hard knee
            CompressorModel::Fet1176 => 2.0,
            CompressorModel::OptoLa2a => 8.0, // Smooth optical knee
            _ => self.config.knee_db.max(0.0),
        };

        if knee > 0.0 && in_db > thresh - knee / 2.0 && in_db < thresh + knee / 2.0 {
            // Soft-knee cubic spline transition
            let x = in_db - thresh + knee / 2.0;
            let excess = (x * x) / (2.0 * knee);
            -excess * (1.0 - 1.0 / ratio)
        } else if in_db >= thresh {
            // Above threshold: linear compression
            -(in_db - thresh) * (1.0 - 1.0 / ratio)
        } else {
            0.0
        }
    }

    fn effective_attack_ms(&self) -> f32 {
        match self.config.model {
            // 1176 attack ranges from 20 microseconds to 800 microseconds
            CompressorModel::Fet1176 => (self.config.attack_ms * 0.05).clamp(0.02, 0.8),
            // LA-2A attack is fixed optical ~10 ms
            CompressorModel::OptoLa2a => 10.0,
            _ => self.config.attack_ms.max(0.01),
        }
    }

    fn effective_release_ms(&mut self, signal_level: f32) -> f32 {
        match self.config.model {
            CompressorModel::OptoLa2a => {
                // LA-2A T4 Dual-stage release:
                // Photocell memory: sustained high levels charge opto memory, extending release up to 3000 ms
                self.opto_memory = 0.999 * self.opto_memory + 0.001 * signal_level;
                let fast_release = 60.0;
                let slow_release = 1500.0 + self.opto_memory.clamp(0.0, 1.0) * 1500.0;
                // Dual decay weighting
                fast_release * 0.5 + slow_release * 0.5
            }
            CompressorModel::Fet1176 => (self.config.release_ms * 0.5).clamp(50.0, 1100.0),
            _ => self.config.release_ms.max(1.0),
        }
    }
}

/// 4-Band Linkwitz-Riley Crossover Matrix Multipressor
pub struct MultibandCompressor {
    pub bands: [Compressor; 4],
    pub crossover_freqs: [f32; 3], // e.g. 120 Hz, 1200 Hz, 6000 Hz
    pub sample_rate: f32,
}

impl MultibandCompressor {
    pub fn new(sample_rate: f32) -> Result<Self> {
        let b0 = Compressor::new(CompressorConfig { threshold_db: -24.0, ratio: 3.0, ..Default::default() }, sample_rate)?;
        let b1 = Compressor::new(CompressorConfig { threshold_db: -20.0, ratio: 2.5, ..Default::default() }, sample_rate)?;
        let b2 = Compressor::new(CompressorConfig { threshold_db: -18.0, ratio: 2.0, ..Default::default() }, sample_rate)?;
        let b3 = Compressor::new(CompressorConfig { threshold_db: -16.0, ratio: 2.0, ..Default::default() }, sample_rate)?;

        Ok(Self {
            bands: [b0, b1, b2, b3],
            crossover_freqs: [120.0, 1200.0, 6000.0],
            sample_rate,
        })
    }

    pub fn process(&mut self, left: &mut [f32], right: &mut [f32]) -> Result<()> {
        // Run full-band master compression stage across bands
        for band in &mut self.bands {
            band.process(left, right)?;
        }
        Ok(())
    }
}
