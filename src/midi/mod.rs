#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MidiDirection {
    Forward,
    Reverse,
    PingPong,
    Random,
}

#[derive(Debug, Clone)]
pub struct SequencerStep {
    pub active: bool,
    pub note_number: u8,
    pub velocity: u16,        // 16-bit high-resolution velocity (MIDI 2.0)
    pub gate_percent: u8,     // 1% to 100%
    pub ratchet: u8,          // Sub-step ratcheting: 1x, 2x, 3x, 4x, ..., 16x
    pub probability: u8,      // 0% to 100% chance
    pub octave_offset: i8,    // -3 to +3
    pub articulation_id: u8,  // Logic Pro Articulation ID (1 to 255)
}

impl Default for SequencerStep {
    fn default() -> Self {
        Self {
            active: true,
            note_number: 60,
            velocity: 32768, // Half scale
            gate_percent: 75,
            ratchet: 1,
            probability: 100,
            octave_offset: 0,
            articulation_id: 1,
        }
    }
}

pub struct StepSequencerLane {
    pub name: String,
    pub steps: Vec<SequencerStep>,
    pub playback_direction: MidiDirection,
    pub current_step: usize,
    ping_pong_forward: bool,
}

impl StepSequencerLane {
    pub fn new(name: String, step_count: usize) -> Self {
        Self {
            name,
            steps: vec![SequencerStep::default(); step_count],
            playback_direction: MidiDirection::Forward,
            current_step: 0,
            ping_pong_forward: true,
        }
    }

    /// Advance step according to playback direction and evaluate active triggers
    pub fn advance_step(&mut self) -> Option<SequencerStep> {
        if self.steps.is_empty() {
            return None;
        }

        let step = self.steps[self.current_step].clone();

        match self.playback_direction {
            MidiDirection::Forward => {
                self.current_step = (self.current_step + 1) % self.steps.len();
            }
            MidiDirection::Reverse => {
                self.current_step = if self.current_step == 0 {
                    self.steps.len() - 1
                } else {
                    self.current_step - 1
                };
            }
            MidiDirection::PingPong => {
                if self.ping_pong_forward {
                    if self.current_step + 1 >= self.steps.len() {
                        self.ping_pong_forward = false;
                        self.current_step = self.steps.len().saturating_sub(2);
                    } else {
                        self.current_step += 1;
                    }
                } else {
                    if self.current_step == 0 {
                        self.ping_pong_forward = true;
                        self.current_step = 1.min(self.steps.len() - 1);
                    } else {
                        self.current_step -= 1;
                    }
                }
            }
            MidiDirection::Random => {
                self.current_step = (self.current_step * 1664525 + 1013904223) % self.steps.len();
            }
        }

        if step.active && step.probability > 0 {
            Some(step)
        } else {
            None
        }
    }
}
