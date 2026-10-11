use crate::error::{LogicError, Result};
use crate::mixing::aux_bus::{AuxBus, MAX_BUSES};
use crate::mixing::channel_strip::ChannelStrip;

pub const MAX_CHANNELS: usize = 1000;

#[derive(Debug, Clone)]
pub struct VcaFader {
    pub id: usize,
    pub name: String,
    pub gain_db: f32,
    pub muted: bool,
    pub assigned_channels: Vec<usize>,
}

impl VcaFader {
    pub fn new(id: usize, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            gain_db: 0.0,
            muted: false,
            assigned_channels: Vec::new(),
        }
    }

    pub fn linear_gain(&self) -> f32 {
        if self.muted || self.gain_db <= -96.0 {
            0.0
        } else {
            10.0f32.powf(self.gain_db / 20.0)
        }
    }
}

pub struct MixingEngine {
    pub channels: Vec<ChannelStrip>,
    pub buses: Vec<AuxBus>,
    pub vca_faders: Vec<VcaFader>,
    pub master_fader_db: f32,
    pub sample_rate: f32,
    pub pdc_enabled: bool,
    pub channel_latencies_samples: Vec<usize>,
    pub low_latency_cutoff_ms: f32,
    pub low_latency_mode: bool,
}

impl MixingEngine {
    pub fn new(sample_rate: f32) -> Result<Self> {
        if sample_rate <= 0.0 {
            return Err(LogicError::InvalidSampleRate(sample_rate));
        }

        let mut buses = Vec::with_capacity(MAX_BUSES);
        for id in 0..MAX_BUSES {
            let name = if id == 0 {
                "Stereo Out".to_string()
            } else {
                format!("Bus {id}")
            };
            buses.push(AuxBus::new(id, name));
        }

        Ok(Self {
            channels: Vec::new(),
            buses,
            vca_faders: Vec::new(),
            master_fader_db: 0.0,
            sample_rate,
            pdc_enabled: true,
            channel_latencies_samples: Vec::new(),
            low_latency_cutoff_ms: 5.0,
            low_latency_mode: false,
        })
    }

    pub fn add_channel(&mut self, name: impl Into<String>) -> Result<usize> {
        if self.channels.len() >= MAX_CHANNELS {
            return Err(LogicError::DspError(
                "Max channel limit reached (1000)".into(),
            ));
        }
        let id = self.channels.len();
        self.channels.push(ChannelStrip::new(id, name));
        self.channel_latencies_samples.push(0);
        Ok(id)
    }

    pub fn add_vca(&mut self, name: impl Into<String>) -> usize {
        let id = self.vca_faders.len();
        self.vca_faders.push(VcaFader::new(id, name));
        id
    }

    pub fn set_channel_latency(&mut self, channel_id: usize, latency_samples: usize) {
        if channel_id < self.channel_latencies_samples.len() {
            self.channel_latencies_samples[channel_id] = latency_samples;
        }
    }

    /// Calculate Plugin Delay Compensation (PDC) delay offsets for all channels
    pub fn calculate_pdc_compensation(&self) -> Vec<usize> {
        if !self.pdc_enabled || self.channel_latencies_samples.is_empty() {
            return vec![0; self.channel_latencies_samples.len()];
        }
        let max_lat = *self.channel_latencies_samples.iter().max().unwrap_or(&0);
        self.channel_latencies_samples
            .iter()
            .map(|&lat| max_lat.saturating_sub(lat))
            .collect()
    }

    /// Master linear gain calculated from fader dB
    pub fn master_linear_gain(&self) -> f32 {
        if self.master_fader_db <= -96.0 {
            0.0
        } else {
            10.0f32.powf(self.master_fader_db / 20.0)
        }
    }

    /// Process mixer graph summing: channels accumulate to assigned bus, buses accumulate to Stereo Out (Bus 0)
    pub fn sum_buses(&mut self, master_left: &mut [f32], master_right: &mut [f32]) {
        let len = master_left.len().min(master_right.len());
        let master_gain = self.master_linear_gain();

        // Accumulate Bus 0 (Stereo Out) into master output
        if let Some(bus0) = self.buses.first() {
            for i in 0..len {
                if i < bus0.left_accumulator.len() {
                    master_left[i] = bus0.left_accumulator[i] * master_gain;
                    master_right[i] = bus0.right_accumulator[i] * master_gain;
                }
            }
        }
    }
}
