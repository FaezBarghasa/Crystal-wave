use crate::error::{LogicError, Result};

pub const PPQN: u64 = 960;

#[derive(Debug, Clone)]
pub struct TempoEvent {
    pub tick: u64,
    pub bpm: f32,
}

#[derive(Debug, Clone)]
pub struct TimeSignatureEvent {
    pub tick: u64,
    pub numerator: u8,
    pub denominator: u8,
}

#[derive(Debug, Clone)]
pub struct Timeline {
    pub tempo_map: Vec<TempoEvent>,
    pub signature_map: Vec<TimeSignatureEvent>,
}

impl Default for Timeline {
    fn default() -> Self {
        Self {
            tempo_map: vec![TempoEvent { tick: 0, bpm: 120.0 }],
            signature_map: vec![TimeSignatureEvent { tick: 0, numerator: 4, denominator: 4 }],
        }
    }
}

impl Timeline {
    pub fn tick_to_seconds(&self, target_tick: u64) -> f64 {
        let mut total_seconds = 0.0;
        let mut prev_tick = 0u64;
        let mut current_bpm = 120.0f32;

        for event in &self.tempo_map {
            if event.tick > target_tick {
                break;
            }
            let delta_ticks = event.tick - prev_tick;
            total_seconds += (delta_ticks as f64) / (PPQN as f64) * (60.0 / current_bpm as f64);
            prev_tick = event.tick;
            current_bpm = event.bpm;
        }

        let rem_ticks = target_tick - prev_tick;
        total_seconds += (rem_ticks as f64) / (PPQN as f64) * (60.0 / current_bpm as f64);
        total_seconds
    }
}

#[derive(Debug, Clone)]
pub struct Take {
    pub id: usize,
    pub name: String,
    pub audio_samples: Vec<f32>,
}

#[derive(Debug, Clone)]
pub struct CompSlice {
    pub take_id: usize,
    pub start_sample: usize,
    pub end_sample: usize,
}

/// Quick Swipe Comping Solver (Logic Pro Take Folder model)
pub struct TakeFolder {
    pub takes: Vec<Take>,
    pub comp_slices: Vec<CompSlice>,
    pub crossfade_samples: usize, // e.g. 5ms at 44.1kHz = ~220 samples
}

impl TakeFolder {
    pub fn new(crossfade_samples: usize) -> Self {
        Self {
            takes: Vec::new(),
            comp_slices: Vec::new(),
            crossfade_samples,
        }
    }

    pub fn add_take(&mut self, name: String, samples: Vec<f32>) -> usize {
        let id = self.takes.len();
        self.takes.push(Take { id, name, audio_samples: samples });
        id
    }

    pub fn set_comp_slice(&mut self, take_id: usize, start_sample: usize, end_sample: usize) -> Result<()> {
        if take_id >= self.takes.len() {
            return Err(LogicError::IncompatibleTakeBoundary {
                slice_tick: start_sample as u64,
                start: 0,
                end: self.takes.len() as u64,
            });
        }
        self.comp_slices.push(CompSlice { take_id, start_sample, end_sample });
        self.comp_slices.sort_by_key(|s| s.start_sample);
        Ok(())
    }

    /// Render final composite audio track with equal-power crossfades at slice boundaries
    pub fn render_comp(&self, total_samples: usize) -> Vec<f32> {
        let mut output = vec![0.0; total_samples];
        let xfade_len = self.crossfade_samples;

        for slice in &self.comp_slices {
            if let Some(take) = self.takes.iter().find(|t| t.id == slice.take_id) {
                let start = slice.start_sample.min(output.len());
                let end = slice.end_sample.min(output.len()).min(take.audio_samples.len());

                for i in start..end {
                    let mut sample = take.audio_samples[i];

                    // Equal-power crossfade at start
                    if i < start + xfade_len && start > 0 {
                        let phase = (i - start) as f32 / xfade_len as f32;
                        let fade_in = (phase * std::f32::consts::FRAC_PI_2).sin();
                        sample *= fade_in;
                    }

                    // Equal-power crossfade at end
                    if i + xfade_len > end && end < output.len() {
                        let phase = (end - i) as f32 / xfade_len as f32;
                        let fade_out = (phase * std::f32::consts::FRAC_PI_2).sin();
                        sample *= fade_out;
                    }

                    output[i] += sample;
                }
            }
        }

        output
    }
}
