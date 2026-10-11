#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginCategory {
    Dynamics,
    Equalizer,
    Filter,
    Reverb,
    Delay,
    Modulation,
    Distortion,
    PitchCorrection,
    Utility,
    Synth,
    Sampler,
    MidiEffect,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PluginDescriptor {
    pub unique_id: u32,
    pub name: String,
    pub vendor: String,
    pub version: String,
    pub category: PluginCategory,
    pub num_inputs: usize,
    pub num_outputs: usize,
    pub supports_sidechain: bool,
    pub latency_samples: usize,
}

impl PluginDescriptor {
    pub fn new(unique_id: u32, name: impl Into<String>, category: PluginCategory) -> Self {
        Self {
            unique_id,
            name: name.into(),
            vendor: "Apple / PureRust".into(),
            version: "12.3.1".into(),
            category,
            num_inputs: 2,
            num_outputs: 2,
            supports_sidechain: false,
            latency_samples: 0,
        }
    }
}
