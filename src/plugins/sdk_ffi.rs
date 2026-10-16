use crate::plugins::plugin_descriptor::PluginCategory;
use std::ffi::{c_char, c_void};

/// Universal C ABI Version for Crystal-Wave Plugin SDK
pub const CRYSTAL_PLUGIN_API_VERSION: u32 = 1;

/// C-compatible plugin descriptor
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CrystalPluginDescriptorC {
    pub api_version: u32,
    pub unique_id: u32,
    pub name: *const c_char,
    pub vendor: *const c_char,
    pub version: *const c_char,
    pub category: u32, // maps to PluginCategory
    pub num_inputs: u32,
    pub num_outputs: u32,
    pub supports_sidechain: bool,
    pub latency_samples: u32,
}

/// C-compatible Audio Buffer specification
#[repr(C)]
pub struct CrystalAudioBufferC {
    pub left_channel: *mut f32,
    pub right_channel: *mut f32,
    pub sidechain_channel: *const f32,
    pub num_samples: u32,
}

/// Function table implemented by any shared object (.so / .dll / .dylib) plugin
#[repr(C)]
pub struct CrystalPluginVTable {
    pub get_descriptor: unsafe extern "C" fn() -> CrystalPluginDescriptorC,
    pub create_instance: unsafe extern "C" fn(sample_rate: f32) -> *mut c_void,
    pub destroy_instance: unsafe extern "C" fn(instance: *mut c_void),
    pub process_audio:
        unsafe extern "C" fn(instance: *mut c_void, buffer: *mut CrystalAudioBufferC),
    pub reset_state: unsafe extern "C" fn(instance: *mut c_void),
    pub get_parameter_count: unsafe extern "C" fn(instance: *mut c_void) -> u32,
    pub get_parameter_value: unsafe extern "C" fn(instance: *mut c_void, param_index: u32) -> f32,
    pub set_parameter_value:
        unsafe extern "C" fn(instance: *mut c_void, param_index: u32, value: f32),
    pub get_parameter_name:
        unsafe extern "C" fn(instance: *mut c_void, param_index: u32) -> *const c_char,

    // Omid Protocol Integration (Fixed 8-byte Packet Control & Haptics)
    pub handle_omid_packet:
        Option<unsafe extern "C" fn(instance: *mut c_void, packet_bytes: *const u8)>,
    pub poll_omid_response:
        Option<unsafe extern "C" fn(instance: *mut c_void, out_packet_bytes: *mut u8) -> bool>,
}

/// Safe helper mapping u32 to PluginCategory
pub fn category_from_u32(val: u32) -> PluginCategory {
    match val {
        0 => PluginCategory::Dynamics,
        1 => PluginCategory::Equalizer,
        2 => PluginCategory::Filter,
        3 => PluginCategory::Reverb,
        4 => PluginCategory::Delay,
        5 => PluginCategory::Modulation,
        6 => PluginCategory::Distortion,
        7 => PluginCategory::PitchCorrection,
        8 => PluginCategory::Utility,
        9 => PluginCategory::Synth,
        10 => PluginCategory::Sampler,
        _ => PluginCategory::MidiEffect,
    }
}
