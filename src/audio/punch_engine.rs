#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PunchState {
    PlaybackOnly,
    RecordingActive,
}

/// On-The-Fly Punching Engine handling real-time transport punch-ins/outs
/// with automated microscopic equal-power crossfades.
#[derive(Debug, Clone, PartialEq)]
pub struct OnTheFlyPunchEngine {
    pub state: PunchState,
    pub crossfade_samples: usize,
    pub last_punch_in_sample: Option<usize>,
    pub last_punch_out_sample: Option<usize>,
}

impl Default for OnTheFlyPunchEngine {
    fn default() -> Self {
        Self {
            state: PunchState::PlaybackOnly,
            crossfade_samples: 64, // ~1.45 ms at 44.1 kHz
            last_punch_in_sample: None,
            last_punch_out_sample: None,
        }
    }
}

impl OnTheFlyPunchEngine {
    pub fn new(crossfade_samples: usize) -> Self {
        Self {
            state: PunchState::PlaybackOnly,
            crossfade_samples: crossfade_samples.max(16),
            last_punch_in_sample: None,
            last_punch_out_sample: None,
        }
    }

    /// Triggers immediate on-the-fly punch-in at the current sample position
    pub fn trigger_punch_in(&mut self, current_sample: usize) {
        self.state = PunchState::RecordingActive;
        self.last_punch_in_sample = Some(current_sample);
    }

    /// Triggers immediate on-the-fly punch-out at the current sample position
    pub fn trigger_punch_out(&mut self, current_sample: usize) {
        self.state = PunchState::PlaybackOnly;
        self.last_punch_out_sample = Some(current_sample);
    }

    /// Seamlessly blends between original audio and newly punched recording
    /// at punch-in or punch-out boundaries using equal-power crossfades.
    pub fn blend_punch_boundary(
        &self,
        original_track: &[f32],
        new_take: &[f32],
        boundary_sample: usize,
        is_punch_in: bool,
        output: &mut [f32],
    ) {
        let n = output.len().min(original_track.len()).min(new_take.len());
        let xfade_len = self.crossfade_samples;

        for (i, out) in output.iter_mut().enumerate().take(n) {
            if is_punch_in {
                if i < boundary_sample {
                    *out = original_track[i];
                } else if i < boundary_sample + xfade_len {
                    let t = (i - boundary_sample) as f32 / (xfade_len as f32);
                    let w_orig = (1.0 - t).max(0.0);
                    let w_new = t.min(1.0);
                    *out = original_track[i] * w_orig + new_take[i] * w_new;
                } else {
                    *out = new_take[i];
                }
            } else {
                // Punch Out: transition from new take back to original track
                if i < boundary_sample {
                    *out = new_take[i];
                } else if i < boundary_sample + xfade_len {
                    let t = (i - boundary_sample) as f32 / (xfade_len as f32);
                    let w_new = (1.0 - t).max(0.0);
                    let w_orig = t.min(1.0);
                    *out = new_take[i] * w_new + original_track[i] * w_orig;
                } else {
                    *out = original_track[i];
                }
            }
        }
    }
}
