use crate::error::{LogicError, Result};
use crate::mixing::aux_bus::{AuxBus, MAX_BUSES};
use crate::mixing::channel_strip::ChannelStrip;

pub const MAX_CHANNELS: usize = 1000;
pub const FL_STUDIO_MAX_INSERT_TRACKS: usize = 125;
pub const MASTER_CLIPPER_CEILING_DB: f32 = -0.1;
pub const MASTER_CLIPPER_THRESH_DB: f32 = -2.0;

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
    pub master_strip: ChannelStrip,
    pub master_soft_clipper_enabled: bool,
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

        let master_strip = ChannelStrip::new(9999, "Master");

        Ok(Self {
            channels: Vec::new(),
            buses,
            vca_faders: Vec::new(),
            master_fader_db: 0.0,
            master_strip,
            master_soft_clipper_enabled: true,
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

    /// Ensure that an insert channel strip exists up to `track_id`, dynamically allocating insert strips as needed
    pub fn ensure_channel_exists(&mut self, track_id: usize) -> Result<()> {
        if track_id >= MAX_CHANNELS {
            return Err(LogicError::DspError(format!(
                "Track ID {track_id} exceeds maximum channel limit ({MAX_CHANNELS})"
            )));
        }
        while self.channels.len() <= track_id {
            let next_id = self.channels.len();
            let name = format!("Insert {}", next_id + 1);
            self.add_channel(name)?;
        }
        Ok(())
    }

    /// Populate up to `count` FL Studio style insert mixer channels (e.g. 125 tracks)
    pub fn init_fl_insert_tracks(&mut self, count: usize) -> Result<()> {
        let target = count.min(MAX_CHANNELS);
        while self.channels.len() < target {
            let next_id = self.channels.len();
            let name = format!("Insert {}", next_id + 1);
            self.add_channel(name)?;
        }
        Ok(())
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

    /// Master linear gain calculated from master strip fader dB
    pub fn master_linear_gain(&self) -> f32 {
        self.master_strip.linear_gain()
    }

    /// Clear all bus accumulators for a new processing block
    pub fn clear_bus_accumulators(&mut self, block_size: usize) {
        for bus in &mut self.buses {
            bus.prepare_buffer(block_size);
        }
    }

    /// Route channel audio into its assigned destination bus with gain, pan, and mute/solo handling
    pub fn route_channel_audio(&mut self, channel_id: usize, left: &[f32], right: &[f32]) {
        if channel_id >= self.channels.len() || self.buses.is_empty() {
            return;
        }

        let any_solo = self.channels.iter().any(|c| c.solo);
        let ch = &self.channels[channel_id];

        if ch.muted || (any_solo && !ch.solo) {
            return;
        }

        let fader_gain = if ch.volume_fader_db <= -96.0 {
            0.0
        } else {
            10.0f32.powf(ch.volume_fader_db / 20.0)
        };

        let vca_gain = if let Some(vca_id) = ch.vca_group {
            self.vca_faders
                .get(vca_id)
                .map(|v| v.linear_gain())
                .unwrap_or(1.0)
        } else {
            1.0
        };

        let total_gain = fader_gain * vca_gain;
        if total_gain <= 0.0 {
            return;
        }

        let pan = ch.pan.balance.clamp(-1.0, 1.0);
        let (pan_l, pan_r) = if pan <= 0.0 {
            (1.0, 1.0 + pan)
        } else {
            (1.0 - pan, 1.0)
        };

        let target_bus_id = ch.output_bus.min(self.buses.len() - 1);
        let len = left.len().min(right.len());

        let target_bus = &mut self.buses[target_bus_id];
        if target_bus.left_accumulator.len() < len {
            target_bus.prepare_buffer(len);
        }

        let mut panned_l = [0.0f32; 128];
        let mut panned_r = [0.0f32; 128];
        for chunk_start in (0..len).step_by(128) {
            let chunk_len = (len - chunk_start).min(128);
            for j in 0..chunk_len {
                panned_l[j] = left[chunk_start + j] * pan_l;
                panned_r[j] = right[chunk_start + j] * pan_r;
            }
            target_bus.accumulate(&panned_l[..chunk_len], &panned_r[..chunk_len], total_gain);
        }
    }

    /// Process mixer graph summing: topological multi-bus summing hierarchically into Bus 0 (Stereo Out), then Master
    pub fn sum_buses(&mut self, master_left: &mut [f32], master_right: &mut [f32]) {
        let len = master_left.len().min(master_right.len());

        if self.buses.is_empty() {
            return;
        }

        // Prepare all active bus buffers to len
        for bus in &mut self.buses {
            if bus.left_accumulator.len() < len {
                bus.prepare_buffer(len);
            }
        }

        // 1. Topological summation: For buses 1..256, if bus has target_bus pointing to another bus > 0,
        // accumulate into that parent bus first. We process in reverse index order.
        for i in (1..self.buses.len()).rev() {
            if let Some(target) = self.buses[i].target_bus
                && target < i
                && target > 0
            {
                let gain = self.buses[i].linear_gain();
                if gain > 0.0 && !self.buses[i].left_accumulator.is_empty() {
                    let (earlier, current) = self.buses.split_at_mut(i);
                    earlier[target].accumulate(
                        &current[0].left_accumulator,
                        &current[0].right_accumulator,
                        gain,
                    );
                }
            }
        }

        // 2. All remaining buses with target_bus == Some(0) accumulate into Bus 0 (Stereo Out)
        let (bus0_slice, other_buses) = self.buses.split_at_mut(1);
        let bus0 = &mut bus0_slice[0];
        for bus in other_buses {
            if bus.target_bus == Some(0) {
                let gain = bus.linear_gain();
                if gain > 0.0 && !bus.left_accumulator.is_empty() {
                    bus0.accumulate(&bus.left_accumulator, &bus.right_accumulator, gain);
                }
            }
        }

        // 3. Accumulate Bus 0 (Stereo Out) into master output
        for i in 0..len {
            if i < bus0.left_accumulator.len() {
                master_left[i] = bus0.left_accumulator[i];
                master_right[i] = bus0.right_accumulator[i];
            } else {
                master_left[i] = 0.0;
                master_right[i] = 0.0;
            }
        }

        // 4. Process Master Strip Inserts, Fader Gain, Polarity, Stereo Separation & Meter Ballistics
        self.master_strip
            .process_block(&mut master_left[..len], &mut master_right[..len], 1.0);

        // 5. Apply Master Soft Clipper (strict -0.1 dBFS ceiling saturation)
        if self.master_soft_clipper_enabled {
            Self::apply_soft_clipper(&mut master_left[..len], &mut master_right[..len]);
        }
    }

    /// Clean-room Fruity Soft Clipper: Smooth hyperbolic saturation curve with strict -0.1 dBFS ceiling
    pub fn apply_soft_clipper(left: &mut [f32], right: &mut [f32]) {
        let ceiling = 10.0f32.powf(MASTER_CLIPPER_CEILING_DB / 20.0);
        let thresh = 10.0f32.powf(MASTER_CLIPPER_THRESH_DB / 20.0);
        let range = ceiling - thresh;
        let len = left.len().min(right.len());

        for i in 0..len {
            let mut l = left[i];
            let mut r = right[i];
            for sample in [&mut l, &mut r] {
                let x = *sample;
                let abs_x = x.abs();
                if abs_x > thresh {
                    let excess = abs_x - thresh;
                    let sat = thresh + range * (excess / range).tanh();
                    *sample = sat.copysign(x);
                }
            }
            left[i] = l;
            right[i] = r;
        }
    }
}
