use crate::mixing::meter::LevelMeter;
use crate::mixing::routing_mode::SidechainRouting;
use crate::mixing::send_node::SendNode;
use crate::mixing::spatial_audio_processor::SpatialObjectPanner;
use crate::plugins::PluginChain;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PanMode {
    #[default]
    Balance,
    DualPan,
    Binaural,
    DolbyAtmos3D,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChannelPan {
    pub mode: PanMode,
    pub balance: f32,    // -1.0 to +1.0
    pub dual_left: f32,  // -1.0 to +1.0
    pub dual_right: f32, // -1.0 to +1.0
    pub object_panner: SpatialObjectPanner,
}

impl Default for ChannelPan {
    fn default() -> Self {
        Self {
            mode: PanMode::Balance,
            balance: 0.0,
            dual_left: -1.0,
            dual_right: 1.0,
            object_panner: SpatialObjectPanner::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelStripState {
    pub volume_fader_db: f32,
    pub pan: ChannelPan,
    pub phase_invert: bool,
    pub delay_offset_samples: i64,
    pub output_bus: usize,
    pub vca_group: Option<usize>,
}

pub struct ChannelStrip {
    pub id: usize,
    pub name: String,
    pub volume_fader_db: f32,
    pub pan: ChannelPan,
    pub muted: bool,
    pub solo: bool,
    pub phase_invert: bool,
    pub delay_offset_samples: i64,
    pub sends: Vec<SendNode>,
    pub sidechain: SidechainRouting,
    pub vca_group: Option<usize>,
    pub output_bus: usize,
    pub meter: LevelMeter,
    pub plugins: PluginChain,
}

impl ChannelStrip {
    pub fn new(id: usize, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            volume_fader_db: 0.0,
            pan: ChannelPan::default(),
            muted: false,
            solo: false,
            phase_invert: false,
            delay_offset_samples: 0,
            sends: Vec::new(),
            sidechain: SidechainRouting::default(),
            vca_group: None,
            output_bus: 0,
            meter: LevelMeter::default(),
            plugins: PluginChain::new(),
        }
    }

    pub fn copy_state(&self) -> ChannelStripState {
        ChannelStripState {
            volume_fader_db: self.volume_fader_db,
            pan: self.pan,
            phase_invert: self.phase_invert,
            delay_offset_samples: self.delay_offset_samples,
            output_bus: self.output_bus,
            vca_group: self.vca_group,
        }
    }

    pub fn paste_state(&mut self, state: &ChannelStripState) {
        self.volume_fader_db = state.volume_fader_db;
        self.pan = state.pan;
        self.phase_invert = state.phase_invert;
        self.delay_offset_samples = state.delay_offset_samples;
        self.output_bus = state.output_bus;
        self.vca_group = state.vca_group;
    }

    pub fn linear_gain(&self) -> f32 {
        if self.muted || self.volume_fader_db <= -96.0 || self.meter.auto_mute_tripped {
            0.0
        } else {
            10.0f32.powf(self.volume_fader_db / 20.0)
        }
    }

    pub fn process_block(&mut self, left: &mut [f32], right: &mut [f32], vca_gain: f32) {
        // 1. Process inserts through plugin chain
        self.plugins.process(left, right, None);

        // 2. Linear gain & phase invert
        let gain = self.linear_gain() * vca_gain;
        let phase_mult = if self.phase_invert { -1.0 } else { 1.0 };

        let len = left.len().min(right.len());
        for i in 0..len {
            left[i] *= gain * phase_mult;
            right[i] *= gain * phase_mult;

            // Apply stereo balance panning
            match self.pan.mode {
                PanMode::Balance => {
                    let bal = self.pan.balance;
                    let l_pan = ((1.0 - bal) * std::f32::consts::FRAC_PI_4).cos();
                    let r_pan = ((1.0 + bal) * std::f32::consts::FRAC_PI_4).cos();
                    left[i] *= l_pan;
                    right[i] *= r_pan;
                }
                PanMode::DualPan => {
                    let l_pan = (self.pan.dual_left + 1.0) * 0.5;
                    let r_pan = (self.pan.dual_right + 1.0) * 0.5;
                    left[i] *= 1.0 - l_pan;
                    right[i] *= r_pan;
                }
                PanMode::Binaural | PanMode::DolbyAtmos3D => {
                    let bed = self.pan.object_panner.calculate_bed_gains();
                    let (b_l, b_r) = self.pan.object_panner.binaural_downmix(&bed);
                    left[i] *= b_l;
                    right[i] *= b_r;
                }
            }
        }

        // Run level metering and safety check
        self.meter.process_block(left, right);
    }
}
