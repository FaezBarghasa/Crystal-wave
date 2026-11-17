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

pub const NUM_INSERT_SLOTS: usize = 10;

#[derive(Debug, Clone, PartialEq)]
pub struct InsertSlot {
    pub slot_id: usize,
    pub plugin_id: Option<u32>,
    pub name: String,
    pub bypassed: bool,
    pub mix: f32, // 0.0 to 1.0 (wet/dry)
}

impl Default for InsertSlot {
    fn default() -> Self {
        Self {
            slot_id: 0,
            plugin_id: None,
            name: "Empty".to_string(),
            bypassed: false,
            mix: 1.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelStripState {
    pub volume_fader_db: f32,
    pub pan: ChannelPan,
    pub stereo_separation: f32,
    pub phase_invert: bool,
    pub swap_channels: bool,
    pub delay_offset_samples: i64,
    pub output_bus: usize,
    pub vca_group: Option<usize>,
    pub slots: [InsertSlot; NUM_INSERT_SLOTS],
}

pub struct ChannelStrip {
    pub id: usize,
    pub name: String,
    pub volume_fader_db: f32,
    pub pan: ChannelPan,
    pub stereo_separation: f32, // -1.0 (100% separated) to +1.0 (100% merged mono)
    pub muted: bool,
    pub solo: bool,
    pub phase_invert: bool,
    pub swap_channels: bool,
    pub delay_offset_samples: i64,
    pub sends: Vec<SendNode>,
    pub sidechain: SidechainRouting,
    pub vca_group: Option<usize>,
    pub output_bus: usize,
    pub meter: LevelMeter,
    pub slots: [InsertSlot; NUM_INSERT_SLOTS],
    pub plugins: PluginChain,
}

impl ChannelStrip {
    pub fn new(id: usize, name: impl Into<String>) -> Self {
        let slots = std::array::from_fn(|i| InsertSlot {
            slot_id: i,
            plugin_id: None,
            name: "Empty".to_string(),
            bypassed: false,
            mix: 1.0,
        });

        Self {
            id,
            name: name.into(),
            volume_fader_db: 0.0,
            pan: ChannelPan::default(),
            stereo_separation: 0.0,
            muted: false,
            solo: false,
            phase_invert: false,
            swap_channels: false,
            delay_offset_samples: 0,
            sends: Vec::new(),
            sidechain: SidechainRouting::default(),
            vca_group: None,
            output_bus: 0,
            meter: LevelMeter::default(),
            slots,
            plugins: PluginChain::new(),
        }
    }

    pub fn copy_state(&self) -> ChannelStripState {
        ChannelStripState {
            volume_fader_db: self.volume_fader_db,
            pan: self.pan,
            stereo_separation: self.stereo_separation,
            phase_invert: self.phase_invert,
            swap_channels: self.swap_channels,
            delay_offset_samples: self.delay_offset_samples,
            output_bus: self.output_bus,
            vca_group: self.vca_group,
            slots: self.slots.clone(),
        }
    }

    pub fn paste_state(&mut self, state: &ChannelStripState) {
        self.volume_fader_db = state.volume_fader_db;
        self.pan = state.pan;
        self.stereo_separation = state.stereo_separation;
        self.phase_invert = state.phase_invert;
        self.swap_channels = state.swap_channels;
        self.delay_offset_samples = state.delay_offset_samples;
        self.output_bus = state.output_bus;
        self.vca_group = state.vca_group;
        self.slots = state.slots.clone();
    }

    pub fn set_stereo_separation(&mut self, sep: f32) {
        self.stereo_separation = sep.clamp(-1.0, 1.0);
    }

    pub fn set_phase_invert(&mut self, invert: bool) {
        self.phase_invert = invert;
    }

    pub fn toggle_phase_invert(&mut self) {
        self.phase_invert = !self.phase_invert;
    }

    pub fn set_swap_channels(&mut self, swap: bool) {
        self.swap_channels = swap;
    }

    pub fn toggle_swap_channels(&mut self) {
        self.swap_channels = !self.swap_channels;
    }

    pub fn set_slot_bypassed(&mut self, slot_id: usize, bypassed: bool) {
        if slot_id < NUM_INSERT_SLOTS {
            self.slots[slot_id].bypassed = bypassed;
            self.plugins.set_bypassed(slot_id, bypassed);
        }
    }

    pub fn set_slot_mix(&mut self, slot_id: usize, mix: f32) {
        if slot_id < NUM_INSERT_SLOTS {
            let mix = mix.clamp(0.0, 1.0);
            self.slots[slot_id].mix = mix;
            self.plugins.set_mix(slot_id, mix);
        }
    }

    pub fn set_slot_name(&mut self, slot_id: usize, name: impl Into<String>) {
        if slot_id < NUM_INSERT_SLOTS {
            self.slots[slot_id].name = name.into();
        }
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

        // 2. Mid/Side Stereo Separation matrixing
        // sigma in [-1.0, 1.0]
        // 0.0 = normal stereo
        // 1.0 = 100% merged (mono: Mid only)
        // -1.0 = 100% separated (side amplified / difference)
        if self.stereo_separation.abs() > 1e-4 {
            let sep = self.stereo_separation.clamp(-1.0, 1.0);
            let len = left.len().min(right.len());
            for i in 0..len {
                let l = left[i];
                let r = right[i];
                let mid = 0.5 * (l + r);
                let side = 0.5 * (l - r);
                let (m_prime, s_prime) = if sep >= 0.0 {
                    (mid, side * (1.0 - sep))
                } else {
                    (mid * (1.0 + sep), side * (1.0 - sep))
                };
                left[i] = m_prime + s_prime;
                right[i] = m_prime - s_prime;
            }
        }

        // 3. Swap Left/Right channels toggle
        if self.swap_channels {
            let len = left.len().min(right.len());
            for i in 0..len {
                std::mem::swap(&mut left[i], &mut right[i]);
            }
        }

        // 4. Linear gain & polarity invert (phase inversion)
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

        // 5. Run level metering and safety check
        self.meter.process_block(left, right);
    }
}
