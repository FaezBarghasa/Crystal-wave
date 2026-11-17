pub mod automation_map;
pub mod automation_object;
pub mod automation_preview;
pub mod aux_bus;
pub mod channel_strip;
pub mod cue_matrix;
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
pub use automation_object::{AutomationNode, AutomationShape, FloatingAutomationObject};
pub use automation_preview::AutomationPreviewManager;
pub use aux_bus::{AuxBus, MAX_BUSES};
pub use channel_strip::{
    ChannelPan, ChannelStrip, ChannelStripState, InsertSlot, NUM_INSERT_SLOTS, PanMode,
};
pub use cue_matrix::{ChannelCueSend, CueMatrix, CueMix, MAX_CUE_MIXES};
pub use meter::LevelMeter;
pub use mixing_engine::{MAX_CHANNELS, MixingEngine, VcaFader};
pub use return_node::ReturnNode;
pub use routing_mode::{ChannelRoutingMode, SidechainRouting};
pub use send_node::{SendNode, SendPosition};
pub use spatial_audio_processor::{BED_CHANNELS_7_1_4, SpatialObjectPanner};
