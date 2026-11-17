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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GraphEditorTarget {
    #[default]
    Velocity,
    Pan,
    Pitch,
    Flam,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GraphEditorState {
    pub open: bool,
    pub channel_id: usize,
    pub target: GraphEditorTarget,
}

#[derive(Debug, Clone)]
pub struct RackChannel {
    pub id: usize,
    pub name: String,
    pub sample_path: Option<String>,
    pub volume: f32,         // 0.0 to 1.25 (0 to 125%)
    pub pan: f32,            // -1.0 to 1.0
    pub pitch_semitones: i8, // -12 to +12 semitones
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
            pitch_semitones: 0,
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

    pub fn set_step_pitch(&mut self, step_index: usize, pitch: i8) {
        if let Some(step) = self.steps.get_mut(step_index) {
            step.pitch_semitones = pitch.clamp(-12, 12);
        }
    }

    pub fn set_step_micro_shift(&mut self, step_index: usize, ticks: u16) {
        if let Some(step) = self.steps.get_mut(step_index) {
            step.micro_shift_ticks = ticks.min(240);
        }
    }

    pub fn set_volume(&mut self, vol: f32) {
        self.volume = vol.clamp(0.0, 1.25);
    }

    pub fn set_pan(&mut self, pan: f32) {
        self.pan = pan.clamp(-1.0, 1.0);
    }

    pub fn set_pitch(&mut self, pitch: i8) {
        self.pitch_semitones = pitch.clamp(-12, 12);
    }

    pub fn set_target_mixer_track(&mut self, track: usize) {
        self.target_mixer_track = track;
    }
}

pub struct ChannelRack {
    pub channels: Vec<RackChannel>,
    pub step_count: usize, // typically 16, 32, 64
    pub graph_editor: GraphEditorState,
    pub global_swing: f32, // 0.0 to 1.0
}

impl ChannelRack {
    pub fn new(step_count: usize) -> Self {
        Self {
            channels: Vec::new(),
            step_count: step_count.max(16),
            graph_editor: GraphEditorState::default(),
            global_swing: 0.0,
        }
    }

    pub fn add_channel(&mut self, name: impl Into<String>) -> usize {
        let id = self.channels.len();
        self.channels
            .push(RackChannel::new(id, name, self.step_count));
        id
    }

    pub fn set_channel_volume(&mut self, channel_id: usize, volume: f32) {
        if let Some(ch) = self.channels.get_mut(channel_id) {
            ch.set_volume(volume);
        }
    }

    pub fn set_channel_pan(&mut self, channel_id: usize, pan: f32) {
        if let Some(ch) = self.channels.get_mut(channel_id) {
            ch.set_pan(pan);
        }
    }

    pub fn set_channel_target_mixer_track(&mut self, channel_id: usize, track: usize) {
        if let Some(ch) = self.channels.get_mut(channel_id) {
            ch.set_target_mixer_track(track);
        }
    }

    pub fn set_channel_pitch(&mut self, channel_id: usize, pitch: i8) {
        if let Some(ch) = self.channels.get_mut(channel_id) {
            ch.set_pitch(pitch);
        }
    }

    pub fn toggle_graph_editor(&mut self, channel_id: usize) {
        if self.graph_editor.open && self.graph_editor.channel_id == channel_id {
            self.graph_editor.open = false;
        } else {
            self.graph_editor.open = true;
            self.graph_editor.channel_id = channel_id;
        }
    }

    pub fn set_graph_target(&mut self, target: GraphEditorTarget) {
        self.graph_editor.target = target;
    }

    pub fn set_step_property(
        &mut self,
        channel_id: usize,
        step_index: usize,
        target: GraphEditorTarget,
        val: f32,
    ) {
        if let Some(ch) = self.channels.get_mut(channel_id) {
            match target {
                GraphEditorTarget::Velocity => ch.set_step_velocity(step_index, val),
                GraphEditorTarget::Pan => ch.set_step_pan(step_index, val),
                GraphEditorTarget::Pitch => ch.set_step_pitch(step_index, val.round() as i8),
                GraphEditorTarget::Flam => ch.set_step_micro_shift(step_index, val.round() as u16),
            }
        }
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
                    let total_pitch = (chan.pitch_semitones as i16) + (step.pitch_semitones as i16);
                    let key = (root_key as i16 + total_pitch).clamp(0, 127) as u8;
                    events.push((start, length, key, step.velocity));
                }
            }
        }
        events
    }
}
