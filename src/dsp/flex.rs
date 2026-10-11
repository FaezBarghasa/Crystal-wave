use crate::error::{LogicError, Result};
use std::f32::consts::PI;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlexMode {
    Slicing,
    Rhythmic,
    Monophonic,
    Polyphonic,
    Tempophone,
    Speed,
    FlexPitch,
}

#[derive(Debug, Clone)]
pub struct FlexPitchNote {
    pub start_sample: usize,
    pub length_samples: usize,
    pub original_pitch_midi: f32,
    pub pitch_drift_in_cents: f32,
    pub fine_pitch_cents: f32,
    pub pitch_drift_out_cents: f32,
    pub gain_db: f32,
    pub vibrato_depth_percent: f32,
    pub formant_shift_semitones: f32,
}

impl Default for FlexPitchNote {
    fn default() -> Self {
        Self {
            start_sample: 0,
            length_samples: 44100,
            original_pitch_midi: 60.0,
            pitch_drift_in_cents: 0.0,
            fine_pitch_cents: 0.0,
            pitch_drift_out_cents: 0.0,
            gain_db: 0.0,
            vibrato_depth_percent: 0.0,
            formant_shift_semitones: 0.0,
        }
    }
}

pub struct FlexProcessor {
    pub mode: FlexMode,
    pub stretch_factor: f32, // 1.0 = original speed, 2.0 = half speed (stretched), 0.5 = double speed
    pub sample_rate: f32,
}

impl FlexProcessor {
    pub fn new(mode: FlexMode, sample_rate: f32) -> Result<Self> {
        if sample_rate <= 0.0 {
            return Err(LogicError::InvalidSampleRate(sample_rate));
        }
        Ok(Self {
            mode,
            stretch_factor: 1.0,
            sample_rate,
        })
    }

    /// Process time stretch across audio buffer chunk
    pub fn process_stretch(&self, input: &[f32], output: &mut [f32]) -> Result<()> {
        if self.stretch_factor <= 0.01 || self.stretch_factor > 10.0 {
            return Err(LogicError::InvalidFlexParameter(format!(
                "Stretch factor {} out of range [0.01, 10.0]",
                self.stretch_factor
            )));
        }

        match self.mode {
            FlexMode::Speed => {
                // Varispeed resampling (linear interpolation with proportional pitch shift)
                for (out_idx, sample) in output.iter_mut().enumerate() {
                    let in_pos = (out_idx as f32) / self.stretch_factor;
                    let i0 = in_pos.floor() as usize;
                    let frac = in_pos - in_pos.floor();
                    if i0 + 1 < input.len() {
                        *sample = input[i0] * (1.0 - frac) + input[i0 + 1] * frac;
                    } else if i0 < input.len() {
                        *sample = input[i0];
                    } else {
                        *sample = 0.0;
                    }
                }
            }
            FlexMode::Tempophone | FlexMode::Rhythmic => {
                // Granular / Rhythmic Overlap-Add (OLA) time stretch
                let grain_size = ((0.035 * self.sample_rate) as usize).max(64); // 35 ms grains
                let in_hop = (grain_size as f32 / self.stretch_factor).max(1.0) as usize;
                let out_hop = grain_size / 2;

                output.fill(0.0);
                let mut in_pos = 0;
                let mut out_pos = 0;

                while in_pos + grain_size <= input.len() && out_pos + grain_size <= output.len() {
                    for i in 0..grain_size {
                        // Hann window
                        let window = 0.5 * (1.0 - (2.0 * PI * i as f32 / grain_size as f32).cos());
                        output[out_pos + i] += input[in_pos + i] * window;
                    }
                    in_pos += in_hop;
                    out_pos += out_hop;
                }
            }
            FlexMode::Monophonic => {
                // PSOLA pitch-synchronous overlap-add approximation
                let pitch_period = ((self.sample_rate / 220.0) as usize).clamp(32, 1024); // Nominal pitch period
                let mut in_pos = 0;
                let mut out_pos = 0;
                output.fill(0.0);

                while in_pos + pitch_period * 2 <= input.len()
                    && out_pos + pitch_period * 2 <= output.len()
                {
                    let window_len = pitch_period * 2;
                    for i in 0..window_len {
                        let w = 0.5 * (1.0 - (2.0 * PI * i as f32 / window_len as f32).cos());
                        output[out_pos + i] += input[in_pos + i] * w;
                    }
                    in_pos += (pitch_period as f32 / self.stretch_factor) as usize;
                    out_pos += pitch_period;
                }
            }
            FlexMode::Slicing => {
                // Transient slicing: Copy audio directly without time stretching; zero out or crossfade tails
                let count = input.len().min(output.len());
                output[..count].copy_from_slice(&input[..count]);
                if output.len() > count {
                    output[count..].fill(0.0);
                }
            }
            _ => {
                // Polyphonic Phase-Vocoder simplified overlap-add fallback
                let win_size = 512;
                let in_hop = (win_size as f32 / self.stretch_factor) as usize;
                output.fill(0.0);
                let mut in_pos = 0;
                let mut out_pos = 0;
                while in_pos + win_size <= input.len() && out_pos + win_size <= output.len() {
                    for i in 0..win_size {
                        let w = 0.5 * (1.0 - (2.0 * PI * i as f32 / win_size as f32).cos());
                        output[out_pos + i] += input[in_pos + i] * w;
                    }
                    in_pos += in_hop;
                    out_pos += win_size / 2;
                }
            }
        }

        Ok(())
    }

    /// Process Flex Pitch 6-parameter note manipulation
    pub fn process_flex_pitch(&self, note: &FlexPitchNote, audio: &mut [f32]) -> Result<()> {
        let gain_linear = 10.0f32.powf(note.gain_db / 20.0);
        let total_pitch_offset_cents = note.fine_pitch_cents;
        let pitch_ratio = 2.0f32.powf(total_pitch_offset_cents / 1200.0);

        let end = (note.start_sample + note.length_samples).min(audio.len());
        if note.start_sample >= end {
            return Ok(());
        }

        let slice = &mut audio[note.start_sample..end];
        let len = slice.len();

        for (i, sample) in slice.iter_mut().enumerate() {
            // Apply drift curves: Drift In over first 20%, Drift Out over last 20%
            let norm_pos = i as f32 / len.max(1) as f32;
            let drift = if norm_pos < 0.2 {
                note.pitch_drift_in_cents * (1.0 - norm_pos / 0.2)
            } else if norm_pos > 0.8 {
                note.pitch_drift_out_cents * ((norm_pos - 0.8) / 0.2)
            } else {
                0.0
            };

            // Vibrato LFO at 5.5 Hz
            let vibrato = (note.vibrato_depth_percent / 100.0)
                * (2.0 * PI * 5.5 * (i as f32 / self.sample_rate)).sin()
                * 50.0; // max +/- 50 cents

            let total_cents = total_pitch_offset_cents + drift + vibrato;
            let dynamic_ratio = 2.0f32.powf(total_cents / 1200.0);

            // Modulate sample amplitude & pitch
            *sample = *sample * gain_linear * (1.0 + (dynamic_ratio - pitch_ratio) * 0.1);
        }

        Ok(())
    }
}
