use crate::elastic::pitch_shift::{FlexPitch6Hotspot, FlexPitchEngine};
use crate::elastic::time_stretch::{granular_stretch, psola_stretch, slicing_stretch};
use crate::error::{LogicError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FlexTimeMode {
    #[default]
    Slicing,
    Rhythmic,
    Monophonic,
    Polyphonic,
    Tempophone,
    Speed,
    FlexPitch,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WarpMarker {
    pub source_sample: usize,
    pub destination_sample: usize,
}

pub struct ElasticAudio {
    pub mode: FlexTimeMode,
    pub sample_rate: f32,
    pub warp_markers: Vec<WarpMarker>,
    pub pitch_notes: Vec<FlexPitch6Hotspot>,
    pub transient_markers: Vec<usize>,
}

impl ElasticAudio {
    pub fn new(mode: FlexTimeMode, sample_rate: f32) -> Result<Self> {
        if sample_rate <= 0.0 {
            return Err(LogicError::InvalidSampleRate(sample_rate));
        }
        Ok(Self {
            mode,
            sample_rate,
            warp_markers: Vec::new(),
            pitch_notes: Vec::new(),
            transient_markers: Vec::new(),
        })
    }

    pub fn add_warp_marker(&mut self, source_sample: usize, destination_sample: usize) {
        self.warp_markers.push(WarpMarker {
            source_sample,
            destination_sample,
        });
        self.warp_markers.sort_by_key(|m| m.destination_sample);
    }

    pub fn delete_warp_marker(&mut self, index: usize) -> bool {
        if index < self.warp_markers.len() {
            self.warp_markers.remove(index);
            true
        } else {
            false
        }
    }

    pub fn add_pitch_note(&mut self, note: FlexPitch6Hotspot) {
        self.pitch_notes.push(note);
    }

    pub fn process(&self, input: &[f32], output: &mut [f32], stretch_factor: f32) -> Result<()> {
        if stretch_factor <= 0.01 || stretch_factor > 10.0 {
            return Err(LogicError::InvalidFlexParameter(format!(
                "Stretch factor {stretch_factor} out of bounds"
            )));
        }

        match self.mode {
            FlexTimeMode::Speed => {
                // Varispeed resampling
                for (out_idx, sample) in output.iter_mut().enumerate() {
                    let in_pos = (out_idx as f32) / stretch_factor;
                    let i0 = in_pos.floor() as usize;
                    let frac = in_pos - in_pos.floor();
                    if i0 + 1 < input.len() {
                        *sample = input[i0] * (1.0 - frac) + input[i0 + 1] * frac;
                    } else if i0 < input.len() {
                        *sample = input[i0];
                    }
                }
            }
            FlexTimeMode::Monophonic => {
                // Approximate pitch period of 100 Hz at current sample rate: e.g. 441 samples
                let period = (self.sample_rate / 150.0).round() as usize;
                psola_stretch(input, output, stretch_factor, period);
            }
            FlexTimeMode::Rhythmic | FlexTimeMode::Tempophone => {
                let grain_size = ((0.035 * self.sample_rate).round() as usize).max(64); // 35ms grain
                let hop_size = grain_size / 4;
                granular_stretch(input, output, stretch_factor, grain_size, hop_size);
            }
            FlexTimeMode::Polyphonic => {
                let grain_size = ((0.050 * self.sample_rate).round() as usize).max(128); // 50ms grain
                let hop_size = grain_size / 4;
                granular_stretch(input, output, stretch_factor, grain_size, hop_size);
            }
            FlexTimeMode::Slicing => {
                slicing_stretch(input, output, &self.transient_markers, stretch_factor);
            }
            FlexTimeMode::FlexPitch => {
                let copy_len = input.len().min(output.len());
                output[..copy_len].copy_from_slice(&input[..copy_len]);
                let pitch_engine = FlexPitchEngine::new(self.sample_rate);
                for note in &self.pitch_notes {
                    pitch_engine.process_note(note, output);
                }
            }
        }

        Ok(())
    }
}
