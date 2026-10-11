#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DitherType {
    None,
    #[default]
    TPDF,       // Flat Triangular Probability Density Function
    Powr1,      // POW-r #1: Flat TPDF noise shaping for spoken word
    Powr2,      // POW-r #2: Low-noise psychoacoustic shaping for acoustic / pop
    Powr3,      // POW-r #3: High-order 9th-degree psychoacoustic noise shaping (2-4 kHz ear curve)
    UV22HR,     // Apogee UV22HR high-frequency Nyquist energy injection
}

pub struct DitherEngine {
    pub dither_type: DitherType,
    prng_state: u64,
    // Error filter memory for high-order noise shaping (POW-r #3)
    error_history: [f32; 10],
}

impl Default for DitherEngine {
    fn default() -> Self {
        Self::new(DitherType::TPDF)
    }
}

impl DitherEngine {
    pub fn new(dither_type: DitherType) -> Self {
        Self {
            dither_type,
            prng_state: 987654321,
            error_history: [0.0; 10],
        }
    }

    fn next_uniform(&mut self) -> f32 {
        self.prng_state = self.prng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
        let val = (self.prng_state >> 32) as u32;
        (val as f32 / u32::MAX as f32) * 2.0 - 1.0
    }

    /// Generates Triangular PDF random noise (-1.0 to +1.0 LSB)
    pub fn generate_tpdf(&mut self) -> f32 {
        let r1 = self.next_uniform();
        let r2 = self.next_uniform();
        (r1 + r2) * 0.5
    }

    /// Process 32-bit float sample and return dithered, quantized integer sample
    pub fn dither_sample_to_i16(&mut self, sample: f32) -> i16 {
        let scale = 32767.0f32;
        let scaled = sample * scale;

        let dither_noise = match self.dither_type {
            DitherType::None => 0.0,
            DitherType::TPDF | DitherType::Powr1 => self.generate_tpdf(),
            DitherType::Powr2 => {
                // Low-noise psychoacoustic shaping: subtract filtered previous error
                let noise = self.generate_tpdf();
                noise - self.error_history[0] * 0.5
            }
            DitherType::Powr3 => {
                // 9th-order FIR noise shaping filter coefficients minimizing 2-4 kHz
                static POWR3_COEFFS: [f32; 9] = [
                    2.412, -3.371, 3.937, -3.424, 2.391, -1.258, 0.499, -0.134, 0.019
                ];
                let mut shaped_err = 0.0f32;
                for i in 0..9 {
                    shaped_err += POWR3_COEFFS[i] * self.error_history[i];
                }
                self.generate_tpdf() + shaped_err
            }
            DitherType::UV22HR => {
                // Apogee UV22HR: injects concentrated high frequency energy near Nyquist
                let r = self.next_uniform();
                r * 0.7 - self.error_history[0] * 0.3
            }
        };

        let target = scaled + dither_noise;
        let quantized = target.round().clamp(-32768.0, 32767.0);
        let error = quantized - target;

        // Shift error history
        for i in (1..10).rev() {
            self.error_history[i] = self.error_history[i - 1];
        }
        self.error_history[0] = error;

        quantized as i16
    }
}
