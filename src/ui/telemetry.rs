use rtrb::{Consumer, Producer, RingBuffer};

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ChannelMeterFrame {
    pub channel_id: usize,
    pub peak_db: f32,
    pub rms_db: f32,
    pub auto_mute_tripped: bool,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct TelemetryBatch {
    pub playhead_tick: u64,
    pub cpu_load_percent: f32,
    pub meters: Vec<ChannelMeterFrame>,
}

pub struct TelemetryChannel;

impl TelemetryChannel {
    pub fn pair(capacity: usize) -> (Producer<TelemetryBatch>, Consumer<TelemetryBatch>) {
        RingBuffer::new(capacity)
    }
}
