pub mod clap_host;
pub mod ffi_guard;
pub mod plugin_chain;
pub mod plugin_descriptor;
pub mod plugin_host;
pub mod plugin_parameters;
pub mod plugin_registry;

pub use clap_host::{ClapBundleScanner, ClapPluginMetadata};
pub use ffi_guard::{HardenedPluginSlot, PluginStatus};
pub use plugin_chain::{InsertSlot, MAX_INSERTS_PER_CHANNEL, PluginChain};
pub use plugin_descriptor::{PluginCategory, PluginDescriptor};
pub use plugin_host::{
    AudioPlugin, NativeChannelEq, NativeChromaVerb, NativeCompressor, NativeTapeDelay,
};
pub use plugin_parameters::PluginParameter;
pub use plugin_registry::{PluginFactory, PluginRegistry, RegisteredPlugin};
