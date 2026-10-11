use std::f32::consts::PI;

pub const FFT_SIZE_1024: usize = 1024;
pub const NUM_BINS_512: usize = 512;

/// Real-Time 1024-point FFT Spectrum Analyzer for Channel EQ & Master Bus visualization
#[derive(Debug, Clone)]
pub struct SpectrumAnalyzer {
    pub window: [f32; FFT_SIZE_1024],
    pub magnitude_db: [f32; NUM_BINS_512],
    pub smoothing: f32, // 0.0 to 1.0 (IIR temporal decay)
}

impl Default for SpectrumAnalyzer {
    fn default() -> Self {
        Self::new(0.7)
    }
}

impl SpectrumAnalyzer {
    pub fn new(smoothing: f32) -> Self {
        let mut window = [0.0f32; FFT_SIZE_1024];
        // Pre-calculate Hann window: w(n) = 0.5 * (1 - cos(2*pi*n / (N - 1)))
        for (i, w) in window.iter_mut().enumerate() {
            *w = 0.5 * (1.0 - (2.0 * PI * i as f32 / (FFT_SIZE_1024 as f32 - 1.0)).cos());
        }

        Self {
            window,
            magnitude_db: [-100.0f32; NUM_BINS_512],
            smoothing: smoothing.clamp(0.0, 0.99),
        }
    }

    /// Processes 1024 audio samples and updates spectral magnitude bins in dB
    pub fn process_frame(&mut self, samples: &[f32; FFT_SIZE_1024]) {
        let mut buffer = [0.0f32; FFT_SIZE_1024];
        for i in 0..FFT_SIZE_1024 {
            buffer[i] = samples[i] * self.window[i];
        }

        // Fast in-place Real FFT using microfft
        let spectrum = microfft::real::rfft_1024(&mut buffer);

        for (i, complex) in spectrum.iter().enumerate() {
            let mag = complex.norm();
            let db = if mag > 1e-5 {
                20.0 * mag.log10()
            } else {
                -100.0
            };

            // Apply temporal exponential smoothing: val = prev * s + current * (1 - s)
            self.magnitude_db[i] =
                self.magnitude_db[i] * self.smoothing + db * (1.0 - self.smoothing);
        }
    }

    /// Converts 512 linear FFT bins into N log-spaced visualization bands (e.g., 32 or 64 bands)
    pub fn get_log_bands(&self, num_bands: usize) -> Vec<f32> {
        let mut bands = vec![-100.0f32; num_bands];
        let max_bin = NUM_BINS_512;

        for (band_idx, band_val) in bands.iter_mut().enumerate() {
            // Logarithmic mapping: bin = min_bin * (max_bin / min_bin)^(band / total)
            let norm_start = band_idx as f32 / num_bands as f32;
            let norm_end = (band_idx + 1) as f32 / num_bands as f32;

            let start_bin = ((norm_start.powf(2.0)) * max_bin as f32).floor() as usize;
            let end_bin = (((norm_end.powf(2.0)) * max_bin as f32).ceil() as usize)
                .clamp(start_bin + 1, max_bin);

            let mut peak_db = -100.0f32;
            for bin in start_bin..end_bin {
                if self.magnitude_db[bin] > peak_db {
                    peak_db = self.magnitude_db[bin];
                }
            }
            *band_val = peak_db;
        }

        bands
    }
}
