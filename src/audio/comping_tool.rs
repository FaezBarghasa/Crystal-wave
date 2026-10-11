use crate::audio::take::Take;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, PartialEq)]
pub struct CompSelection {
    pub take_id: usize,
    pub start_sample: usize,
    pub end_sample: usize,
}

pub struct CompingTool {
    pub takes: Vec<Arc<Mutex<Take>>>,
    pub selections: Vec<CompSelection>,
    pub crossfade_samples: usize,
}

impl CompingTool {
    pub fn new(crossfade_samples: usize) -> Self {
        Self {
            takes: Vec::new(),
            selections: Vec::new(),
            crossfade_samples,
        }
    }

    pub fn with_default_crossfade(sample_rate: f32) -> Self {
        // Default 5 ms equal-power crossfade: 0.005 * sample_rate
        let xfade = (0.005 * sample_rate).round() as usize;
        Self::new(xfade.max(64))
    }

    pub fn add_take(&mut self, take: Arc<Mutex<Take>>) {
        self.takes.push(take);
    }

    pub fn add_selection(&mut self, take_id: usize, start_sample: usize, end_sample: usize) {
        // Quick swipe replace overlapping parts of existing selections
        self.selections
            .retain(|s| s.end_sample <= start_sample || s.start_sample >= end_sample);
        self.selections.push(CompSelection {
            take_id,
            start_sample,
            end_sample,
        });
        self.selections.sort_by_key(|s| s.start_sample);
    }

    pub fn flatten(&self, total_samples: usize) -> Vec<f32> {
        let mut out = vec![0.0f32; total_samples];
        let xfade = self.crossfade_samples;

        for sel in &self.selections {
            if let Some(take_arc) = self.takes.iter().find(|t| {
                if let Ok(t_guard) = t.lock() {
                    t_guard.id == sel.take_id
                } else {
                    false
                }
            }) && let Ok(t_guard) = take_arc.lock()
                && let Ok(buf) = t_guard.buffer.lock()
            {
                let start = sel.start_sample.min(total_samples);
                let end = sel.end_sample.min(total_samples).min(buf.len());
                for i in start..end {
                    let mut sample = buf[i];
                    // Equal-power sinusoidal crossfade in
                    if i < start + xfade && start > 0 {
                        let phase = (i - start) as f32 / xfade as f32;
                        sample *= (phase * std::f32::consts::FRAC_PI_2).sin();
                    }
                    // Equal-power sinusoidal crossfade out
                    if i + xfade > end && end < total_samples {
                        let phase = (end - i) as f32 / xfade as f32;
                        sample *= (phase * std::f32::consts::FRAC_PI_2).sin();
                    }
                    out[i] += sample;
                }
            }
        }
        out
    }

    pub fn flatten_and_merge(&self, total_samples: usize) -> Vec<f32> {
        self.flatten(total_samples)
    }

    pub fn export_comp_to_track(&self, total_samples: usize) -> Take {
        let audio = self.flatten(total_samples);
        Take::new(9999, "Comp Master", audio)
    }
}
