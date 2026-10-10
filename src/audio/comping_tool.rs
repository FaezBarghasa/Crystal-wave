use std::sync::{Arc, Mutex};
use crate::audio::take::Take;

#[derive(Debug, Clone)]
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

    pub fn add_take(&mut self, take: Arc<Mutex<Take>>) {
        self.takes.push(take);
    }

    pub fn add_selection(&mut self, take_id: usize, start_sample: usize, end_sample: usize) {
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
            }) {
                if let Ok(t_guard) = take_arc.lock() {
                    if let Ok(buf) = t_guard.buffer.lock() {
                        let start = sel.start_sample.min(total_samples);
                        let end = sel.end_sample.min(total_samples).min(buf.len());
                        for i in start..end {
                            let mut sample = buf[i];
                            if i < start + xfade && start > 0 {
                                let phase = (i - start) as f32 / xfade as f32;
                                sample *= (phase * std::f32::consts::FRAC_PI_2).sin();
                            }
                            if i + xfade > end && end < total_samples {
                                let phase = (end - i) as f32 / xfade as f32;
                                sample *= (phase * std::f32::consts::FRAC_PI_2).sin();
                            }
                            out[i] += sample;
                        }
                    }
                }
            }
        }
        out
    }
}
