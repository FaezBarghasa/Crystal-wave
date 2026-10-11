#[derive(Debug, Clone, PartialEq)]
pub struct StepProperties {
    pub active: bool,
    pub velocity: f32,          // 0.0 to 1.0 (Graph Editor)
    pub pan: f32,               // -1.0 to 1.0 (Graph Editor)
    pub pitch_semitones: i8,    // -12 to +12 (Graph Editor per-step tune)
    pub micro_shift_ticks: u16, // Flam / micro-timing
}

impl Default for StepProperties {
    fn default() -> Self {
        Self {
            active: false,
            velocity: 0.78,
            pan: 0.0,
            pitch_semitones: 0,
            micro_shift_ticks: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RackChannel {
    pub id: usize,
    pub name: String,
    pub sample_path: Option<String>,
    pub volume: f32, // 0.0 to 1.0
    pub pan: f32,    // -1.0 to 1.0
    pub target_mixer_track: usize,
    pub steps: Vec<StepProperties>,
}

impl RackChannel {
    pub fn new(id: usize, name: impl Into<String>, num_steps: usize) -> Self {
        Self {
            id,
            name: name.into(),
            sample_path: None,
            volume: 0.8,
            pan: 0.0,
            target_mixer_track: id + 1,
            steps: vec![StepProperties::default(); num_steps],
        }
    }

    pub fn toggle_step(&mut self, step_index: usize) -> bool {
        if let Some(step) = self.steps.get_mut(step_index) {
            step.active = !step.active;
            step.active
        } else {
            false
        }
    }

    pub fn set_step_velocity(&mut self, step_index: usize, velocity: f32) {
        if let Some(step) = self.steps.get_mut(step_index) {
            step.velocity = velocity.clamp(0.0, 1.0);
        }
    }

    pub fn set_step_pan(&mut self, step_index: usize, pan: f32) {
        if let Some(step) = self.steps.get_mut(step_index) {
            step.pan = pan.clamp(-1.0, 1.0);
        }
    }
}

pub struct ChannelRack {
    pub channels: Vec<RackChannel>,
    pub step_count: usize, // typically 16, 32, 64
}

impl ChannelRack {
    pub fn new(step_count: usize) -> Self {
        Self {
            channels: Vec::new(),
            step_count: step_count.max(16),
        }
    }

    pub fn add_channel(&mut self, name: impl Into<String>) -> usize {
        let id = self.channels.len();
        self.channels
            .push(RackChannel::new(id, name, self.step_count));
        id
    }

    pub fn toggle_step(&mut self, channel_id: usize, step_index: usize) -> bool {
        if let Some(chan) = self.channels.get_mut(channel_id) {
            chan.toggle_step(step_index)
        } else {
            false
        }
    }

    /// Converts active steps to MIDI note events for pattern playback (at 960 PPQN: 1 step = 240 ticks)
    pub fn to_midi_events(&self, channel_id: usize, root_key: u8) -> Vec<(u64, u64, u8, f32)> {
        let step_ticks = 240u64; // 16th note in 960 PPQN
        let mut events = Vec::new();

        if let Some(chan) = self.channels.get(channel_id) {
            for (i, step) in chan.steps.iter().enumerate() {
                if step.active {
                    let start = (i as u64) * step_ticks + (step.micro_shift_ticks as u64);
                    let length = step_ticks.saturating_sub(10);
                    let key = (root_key as i16 + step.pitch_semitones as i16).clamp(0, 127) as u8;
                    events.push((start, length, key, step.velocity));
                }
            }
        }
        events
    }
}
