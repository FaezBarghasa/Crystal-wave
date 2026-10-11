pub mod automation_map;
pub mod aux_bus;
pub mod channel_strip;
pub mod meter;
pub mod mixing_engine;
pub mod return_node;
pub mod routing_mode;
pub mod send_node;
pub mod spatial_audio_processor;

pub use automation_map::{
    AutomationFollowsRegions, AutomationLane, AutomationMode, AutomationPoint,
    AutomationQuickAccess, AutomationScope,
};
pub use aux_bus::{AuxBus, MAX_BUSES};
pub use channel_strip::{ChannelPan, ChannelStrip, ChannelStripState, PanMode};
pub use meter::LevelMeter;
pub use mixing_engine::{MixingEngine, VcaFader, MAX_CHANNELS};
pub use return_node::ReturnNode;
pub use routing_mode::{ChannelRoutingMode, SidechainRouting};
pub use send_node::{SendNode, SendPosition};
pub use spatial_audio_processor::{SpatialObjectPanner, BED_CHANNELS_7_1_4};
