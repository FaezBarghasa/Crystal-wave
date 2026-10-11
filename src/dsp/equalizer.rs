use crate::error::{LogicError, Result};
use std::f32::consts::PI;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterType {
    LowCut,
    LowShelf,
    Peak,
    HighShelf,
    HighCut,
}

#[derive(Debug, Clone)]
pub struct BiquadCoeffs {
    pub b0: f32,
    pub b1: f32,
    pub b2: f32,
    pub a1: f32,
    pub a2: f32,
}

impl Default for BiquadCoeffs {
    fn default() -> Self {
        Self {
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
        }
    }
}

impl BiquadCoeffs {
    pub fn calculate(
        filter_type: FilterType,
        freq_hz: f32,
        gain_db: f32,
        q: f32,
        sample_rate: f32,
    ) -> Self {
        let w0 = 2.0 * PI * (freq_hz / sample_rate).clamp(1e-4, 0.499);
        let cos_w0 = w0.cos();
        let sin_w0 = w0.sin();
        let alpha = sin_w0 / (2.0 * q.max(0.05));
        let a = 10.0f32.powf(gain_db / 40.0); // sqrt of linear gain for shelving/peaking

        let (b0, b1, b2, a0, a1, a2) = match filter_type {
            FilterType::LowCut => {
                let b0 = (1.0 + cos_w0) / 2.0;
                let b1 = -(1.0 + cos_w0);
                let b2 = (1.0 + cos_w0) / 2.0;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_w0;
                let a2 = 1.0 - alpha;
                (b0, b1, b2, a0, a1, a2)
            }
            FilterType::HighCut => {
                let b0 = (1.0 - cos_w0) / 2.0;
                let b1 = 1.0 - cos_w0;
                let b2 = (1.0 - cos_w0) / 2.0;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_w0;
                let a2 = 1.0 - alpha;
                (b0, b1, b2, a0, a1, a2)
            }
            FilterType::Peak => {
                let b0 = 1.0 + alpha * a;
                let b1 = -2.0 * cos_w0;
                let b2 = 1.0 - alpha * a;
                let a0 = 1.0 + alpha / a;
                let a1 = -2.0 * cos_w0;
                let a2 = 1.0 - alpha / a;
                (b0, b1, b2, a0, a1, a2)
            }
            FilterType::LowShelf => {
                let b0 = a * ((a + 1.0) - (a - 1.0) * cos_w0 + 2.0 * a.sqrt() * alpha);
                let b1 = 2.0 * a * ((a - 1.0) - (a + 1.0) * cos_w0);
                let b2 = a * ((a + 1.0) - (a - 1.0) * cos_w0 - 2.0 * a.sqrt() * alpha);
                let a0 = (a + 1.0) + (a - 1.0) * cos_w0 + 2.0 * a.sqrt() * alpha;
                let a1 = -2.0 * ((a - 1.0) + (a + 1.0) * cos_w0);
                let a2 = (a + 1.0) + (a - 1.0) * cos_w0 - 2.0 * a.sqrt() * alpha;
                (b0, b1, b2, a0, a1, a2)
            }
            FilterType::HighShelf => {
                let b0 = a * ((a + 1.0) + (a - 1.0) * cos_w0 + 2.0 * a.sqrt() * alpha);
                let b1 = -2.0 * a * ((a - 1.0) + (a + 1.0) * cos_w0);
                let b2 = a * ((a + 1.0) + (a - 1.0) * cos_w0 - 2.0 * a.sqrt() * alpha);
                let a0 = (a + 1.0) - (a - 1.0) * cos_w0 + 2.0 * a.sqrt() * alpha;
                let a1 = 2.0 * ((a - 1.0) - (a + 1.0) * cos_w0);
                let a2 = (a + 1.0) - (a - 1.0) * cos_w0 - 2.0 * a.sqrt() * alpha;
                (b0, b1, b2, a0, a1, a2)
            }
        };

        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
        }
    }

    /// Calculate magnitude response in dB at a target frequency
    pub fn magnitude_db(&self, freq_hz: f32, sample_rate: f32) -> f32 {
        let w = 2.0 * PI * (freq_hz / sample_rate);
        let cos_w = w.cos();
        let cos_2w = (2.0 * w).cos();
        let sin_w = w.sin();
        let sin_2w = (2.0 * w).sin();

        let num_re = self.b0 + self.b1 * cos_w + self.b2 * cos_2w;
        let num_im = -self.b1 * sin_w - self.b2 * sin_2w;
        let den_re = 1.0 + self.a1 * cos_w + self.a2 * cos_2w;
        let den_im = -self.a1 * sin_w - self.a2 * sin_2w;

        let num_sq = num_re * num_re + num_im * num_im;
        let den_sq = den_re * den_re + den_im * den_im;

        10.0 * (num_sq / den_sq.max(1e-12)).log10()
    }
}

/// Direct Form II Transposed Biquad filter state
#[derive(Debug, Clone, Default)]
pub struct BiquadState {
    pub s1_l: f32,
    pub s2_l: f32,
    pub s1_r: f32,
    pub s2_r: f32,
}

impl BiquadState {
    #[inline(always)]
    pub fn process_stereo(&mut self, in_l: f32, in_r: f32, c: &BiquadCoeffs) -> (f32, f32) {
        let out_l = c.b0 * in_l + self.s1_l;
        self.s1_l = c.b1 * in_l - c.a1 * out_l + self.s2_l;
        self.s2_l = c.b2 * in_l - c.a2 * out_l;

        let out_r = c.b0 * in_r + self.s1_r;
        self.s1_r = c.b1 * in_r - c.a1 * out_r + self.s2_r;
        self.s2_r = c.b2 * in_r - c.a2 * out_r;

        (out_l, out_r)
    }

    pub fn reset(&mut self) {
        self.s1_l = 0.0;
        self.s2_l = 0.0;
        self.s1_r = 0.0;
        self.s2_r = 0.0;
    }
}

#[derive(Debug, Clone)]
pub struct EqBand {
    pub filter_type: FilterType,
    pub freq_hz: f32,
    pub gain_db: f32,
    pub q: f32,
    pub enabled: bool,
    coeffs: BiquadCoeffs,
}

impl EqBand {
    pub fn new(
        filter_type: FilterType,
        freq_hz: f32,
        gain_db: f32,
        q: f32,
        sample_rate: f32,
    ) -> Self {
        let coeffs = BiquadCoeffs::calculate(filter_type, freq_hz, gain_db, q, sample_rate);
        Self {
            filter_type,
            freq_hz,
            gain_db,
            q,
            enabled: true,
            coeffs,
        }
    }

    pub fn update(&mut self, sample_rate: f32) {
        self.coeffs = BiquadCoeffs::calculate(
            self.filter_type,
            self.freq_hz,
            self.gain_db,
            self.q,
            sample_rate,
        );
    }
}

/// 8-Band Parametric Channel EQ matching Logic Pro Channel EQ topology
pub struct ChannelEq {
    pub bands: [EqBand; 8],
    states: [BiquadState; 8],
    sample_rate: f32,
}

impl ChannelEq {
    pub fn new(sample_rate: f32) -> Result<Self> {
        if sample_rate <= 0.0 {
            return Err(LogicError::InvalidSampleRate(sample_rate));
        }

        let bands = [
            EqBand::new(FilterType::LowCut, 30.0, 0.0, 0.707, sample_rate),
            EqBand::new(FilterType::LowShelf, 100.0, 0.0, 0.707, sample_rate),
            EqBand::new(FilterType::Peak, 250.0, 0.0, 1.0, sample_rate),
            EqBand::new(FilterType::Peak, 750.0, 0.0, 1.0, sample_rate),
            EqBand::new(FilterType::Peak, 2500.0, 0.0, 1.0, sample_rate),
            EqBand::new(FilterType::Peak, 6000.0, 0.0, 1.0, sample_rate),
            EqBand::new(FilterType::HighShelf, 10000.0, 0.0, 0.707, sample_rate),
            EqBand::new(FilterType::HighCut, 18000.0, 0.0, 0.707, sample_rate),
        ];

        Ok(Self {
            bands,
            states: Default::default(),
            sample_rate,
        })
    }

    pub fn process(&mut self, left: &mut [f32], right: &mut [f32]) -> Result<()> {
        if left.len() != right.len() {
            return Err(LogicError::BufferLengthMismatch {
                expected: left.len(),
                actual: right.len(),
            });
        }

        for i in 0..left.len() {
            let mut l = left[i];
            let mut r = right[i];

            for band_idx in 0..8 {
                if self.bands[band_idx].enabled {
                    let (ol, or) =
                        self.states[band_idx].process_stereo(l, r, &self.bands[band_idx].coeffs);
                    l = ol;
                    r = or;
                }
            }

            left[i] = l;
            right[i] = r;
        }

        Ok(())
    }

    /// Calculate combined 8-band frequency response at freq_hz for real-time graphical display
    pub fn total_magnitude_db(&self, freq_hz: f32) -> f32 {
        let mut total = 0.0;
        for band in &self.bands {
            if band.enabled {
                total += band.coeffs.magnitude_db(freq_hz, self.sample_rate);
            }
        }
        total
    }

    pub fn reset(&mut self) {
        for state in &mut self.states {
            state.reset();
        }
    }
}
