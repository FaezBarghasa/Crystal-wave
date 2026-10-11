#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ChannelRoutingMode {
    Mono,
    #[default]
    Stereo,
    StereoSplit,
    Surround714, // 7.1.4 Dolby Atmos Bed
    AmbisonicsBFormat,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SidechainRouting {
    pub enabled: bool,
    pub source_channel_id: usize,
    pub highpass_hz: f32,
    pub lowpass_hz: f32,
    pub listen_key: bool,
}

impl Default for SidechainRouting {
    fn default() -> Self {
        Self {
            enabled: false,
            source_channel_id: 0,
            highpass_hz: 20.0,
            lowpass_hz: 20000.0,
            listen_key: false,
        }
    }
}
