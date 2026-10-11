pub const MAX_CUE_MIXES: usize = 4;

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelCueSend {
    pub level_db: f32,
    pub pan: f32, // -1.0 to 1.0
    pub pre_fader: bool,
    pub muted: bool,
}

impl Default for ChannelCueSend {
    fn default() -> Self {
        Self {
            level_db: -6.0,
            pan: 0.0,
            pre_fader: true,
            muted: false,
        }
    }
}

/// A dedicated zero-latency cue mix output for studio monitoring (e.g., Drummer vs Vocalist)
#[derive(Debug, Clone, PartialEq)]
pub struct CueMix {
    pub id: usize,
    pub name: String,
    pub master_gain_db: f32,
    pub click_enabled: bool,
    pub click_gain_db: f32,
    pub talkback_enabled: bool,
}

impl CueMix {
    pub fn new(id: usize, name: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
            master_gain_db: 0.0,
            click_enabled: false,
            click_gain_db: -3.0,
            talkback_enabled: false,
        }
    }
}

/// Independent Zero-Latency Cue Matrix managing performer headphone feeds
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CueMatrix {
    pub cue_mixes: Vec<CueMix>,
    /// Table of sends: `channel_sends[channel_idx][cue_idx]`
    pub channel_sends: Vec<Vec<ChannelCueSend>>,
}

impl CueMatrix {
    pub fn new(num_channels: usize) -> Self {
        let mut cue_mixes = Vec::new();
        cue_mixes.push(CueMix::new(0, "Cue A (Drummer - Click Heavy)"));
        cue_mixes.push(CueMix::new(1, "Cue B (Vocalist - Reverb Lush)"));
        cue_mixes[0].click_enabled = true;

        let channel_sends = vec![vec![ChannelCueSend::default(); MAX_CUE_MIXES]; num_channels];

        Self {
            cue_mixes,
            channel_sends,
        }
    }

    pub fn set_send_level(
        &mut self,
        channel_idx: usize,
        cue_idx: usize,
        level_db: f32,
    ) -> Result<(), &'static str> {
        if channel_idx < self.channel_sends.len() && cue_idx < MAX_CUE_MIXES {
            self.channel_sends[channel_idx][cue_idx].level_db = level_db;
            Ok(())
        } else {
            Err("IndexOutOfBounds")
        }
    }

    /// Renders a cue mix stereo block by summing channel inputs with send levels and panning.
    pub fn render_cue_mix(
        &self,
        cue_idx: usize,
        channel_buffers: &[(&[f32], &[f32])],
        click_buffer: &[f32],
        out_left: &mut [f32],
        out_right: &mut [f32],
    ) {
        out_left.fill(0.0);
        out_right.fill(0.0);

        if cue_idx >= self.cue_mixes.len() {
            return;
        }

        let cue = &self.cue_mixes[cue_idx];
        let master_gain = 10.0f32.powf(cue.master_gain_db / 20.0);

        let n = out_left.len();

        for (ch_idx, &(l_buf, r_buf)) in channel_buffers.iter().enumerate() {
            if ch_idx >= self.channel_sends.len() {
                break;
            }
            let send = &self.channel_sends[ch_idx][cue_idx];
            if send.muted {
                continue;
            }

            let send_gain = 10.0f32.powf(send.level_db / 20.0);
            let pan_l = ((1.0 - send.pan) * 0.5).clamp(0.0, 1.0);
            let pan_r = ((1.0 + send.pan) * 0.5).clamp(0.0, 1.0);

            let block_len = n.min(l_buf.len()).min(r_buf.len());
            for i in 0..block_len {
                out_left[i] += l_buf[i] * send_gain * pan_l;
                out_right[i] += r_buf[i] * send_gain * pan_r;
            }
        }

        // Add click track if enabled for this performer
        if cue.click_enabled && !click_buffer.is_empty() {
            let click_gain = 10.0f32.powf(cue.click_gain_db / 20.0);
            let block_len = n.min(click_buffer.len());
            for i in 0..block_len {
                out_left[i] += click_buffer[i] * click_gain;
                out_right[i] += click_buffer[i] * click_gain;
            }
        }

        // Apply master cue gain
        for i in 0..n {
            out_left[i] *= master_gain;
            out_right[i] *= master_gain;
        }
    }
}
