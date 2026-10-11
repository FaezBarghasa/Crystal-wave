/// Tape-style audio scrub engine simulating physical tape reel rocking.
/// Plays audio forwards or backwards at variable speeds proportional to mouse velocity.
#[derive(Debug, Clone, PartialEq)]
pub struct TapeScrubEngine {
    pub current_sample_pos: f64,
    pub sample_rate: f32,
    pub pixels_per_second: f32,
    pub max_speed_factor: f32,
}

impl TapeScrubEngine {
    pub fn new(sample_rate: f32, pixels_per_second: f32) -> Self {
        Self {
            current_sample_pos: 0.0,
            sample_rate: sample_rate.max(8000.0),
            pixels_per_second: pixels_per_second.max(10.0),
            max_speed_factor: 4.0, // up to 4x playback speed
        }
    }

    /// Computes the tape speed factor from mouse delta and elapsed time.
    /// Positive = forward play; Negative = reverse play.
    pub fn calculate_scrub_speed(&self, delta_x_pixels: f32, delta_time_secs: f32) -> f32 {
        if delta_time_secs <= 1e-4 {
            return 0.0;
        }
        let px_velocity = delta_x_pixels / delta_time_secs;
        let speed_ratio = px_velocity / self.pixels_per_second;
        speed_ratio.clamp(-self.max_speed_factor, self.max_speed_factor)
    }

    /// Renders a scrub audio block with fractional linear interpolation.
    pub fn render_scrub_block(
        &mut self,
        source_audio: &[f32],
        delta_x_pixels: f32,
        delta_time_secs: f32,
        output_buffer: &mut [f32],
    ) -> f32 {
        if source_audio.is_empty() || output_buffer.is_empty() {
            return 0.0;
        }

        let speed = self.calculate_scrub_speed(delta_x_pixels, delta_time_secs);
        if speed.abs() < 1e-3 {
            output_buffer.fill(0.0);
            return 0.0;
        }

        let step = speed as f64;
        let source_len = source_audio.len();

        for sample in output_buffer.iter_mut() {
            let pos = self.current_sample_pos;
            if pos < 0.0 || pos >= (source_len - 1) as f64 {
                *sample = 0.0;
            } else {
                let idx0 = pos.floor() as usize;
                let idx1 = (idx0 + 1).min(source_len - 1);
                let frac = (pos - (idx0 as f64)) as f32;

                *sample = source_audio[idx0] * (1.0 - frac) + source_audio[idx1] * frac;
            }

            self.current_sample_pos = (self.current_sample_pos + step).clamp(0.0, (source_len - 1) as f64);
        }

        speed
    }
}
