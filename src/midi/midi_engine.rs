use crate::midi::midi_note::MidiNote;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MidiDirection {
    #[default]
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
            steps: vec![SequencerStep::default(); step_count.clamp(1, 64)],
            playback_direction: MidiDirection::Forward,
            current_step: 0,
            ping_pong_forward: true,
        }
    }

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LiveLoopsCellState {
    #[default]
    Empty,
    Stopped,
    Queued,
    Playing,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LiveLoopsCell {
    pub id: usize,
    pub name: String,
    pub length_ticks: u64,
    pub notes: Vec<MidiNote>,
    pub state: LiveLoopsCellState,
}

impl LiveLoopsCell {
    pub fn new(id: usize, name: impl Into<String>, length_ticks: u64) -> Self {
        Self {
            id,
            name: name.into(),
            length_ticks,
            notes: Vec::new(),
            state: LiveLoopsCellState::Stopped,
        }
    }
}

pub struct LiveLoopsGrid {
    pub columns: usize, // Scenes
    pub rows: usize,    // Tracks
    pub cells: Vec<Vec<LiveLoopsCell>>,
}

impl LiveLoopsGrid {
    pub fn new(rows: usize, columns: usize) -> Self {
        let mut cells = Vec::with_capacity(rows);
        let mut id = 0;
        for r in 0..rows {
            let mut row = Vec::with_capacity(columns);
            for c in 0..columns {
                row.push(LiveLoopsCell::new(id, format!("Cell R{r} C{c}"), 3840));
                id += 1;
            }
            cells.push(row);
        }

        Self {
            columns,
            rows,
            cells,
        }
    }

    pub fn trigger_cell(&mut self, row: usize, col: usize) {
        if row < self.rows && col < self.columns {
            // Stop other cells in the same track
            for c in 0..self.columns {
                if self.cells[row][c].state == LiveLoopsCellState::Playing {
                    self.cells[row][c].state = LiveLoopsCellState::Stopped;
                }
            }
            self.cells[row][col].state = LiveLoopsCellState::Playing;
        }
    }

    pub fn trigger_scene(&mut self, col: usize) {
        if col < self.columns {
            for r in 0..self.rows {
                self.trigger_cell(r, col);
            }
        }
    }
}

/// Capture Recording (Retroactive MIDI Record)
pub struct RetroactiveMidiCapture {
    buffer: Vec<MidiNote>,
    capacity: usize,
}

impl RetroactiveMidiCapture {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
            capacity,
        }
    }

    pub fn record_event(&mut self, note: MidiNote) {
        if self.buffer.len() >= self.capacity {
            self.buffer.remove(0);
        }
        self.buffer.push(note);
    }

    /// Retrieve captured notes played while transport was stopped
    pub fn capture_recording(&mut self) -> Vec<MidiNote> {
        let captured = self.buffer.clone();
        self.buffer.clear();
        captured
    }
}
