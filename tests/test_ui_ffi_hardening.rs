use Crystal_wave::plugins::ffi_guard::{HardenedPluginSlot, PluginStatus};
use Crystal_wave::plugins::plugin_descriptor::{PluginCategory, PluginDescriptor};
use Crystal_wave::plugins::plugin_host::AudioPlugin;
use Crystal_wave::plugins::plugin_parameters::PluginParameter;

struct CrashingMockPlugin {
    descriptor: PluginDescriptor,
}

impl CrashingMockPlugin {
    fn new() -> Self {
        Self {
            descriptor: PluginDescriptor::new(9999, "Crashing VST", PluginCategory::Utility),
        }
    }
}

impl AudioPlugin for CrashingMockPlugin {
    fn descriptor(&self) -> &PluginDescriptor {
        &self.descriptor
    }

    fn process(&mut self, _left: &mut [f32], _right: &mut [f32], _sidechain: Option<&[f32]>) {
        panic!("FATAL: Segmentation fault in external third-party C++ VST plugin!");
    }

    fn reset(&mut self) {}

    fn parameters(&self) -> &[PluginParameter] {
        &[]
    }
}

struct RogueNanMockPlugin {
    descriptor: PluginDescriptor,
}

impl RogueNanMockPlugin {
    fn new() -> Self {
        Self {
            descriptor: PluginDescriptor::new(9998, "Rogue NaN Plugin", PluginCategory::Utility),
        }
    }
}

impl AudioPlugin for RogueNanMockPlugin {
    fn descriptor(&self) -> &PluginDescriptor {
        &self.descriptor
    }

    fn process(&mut self, left: &mut [f32], right: &mut [f32], _sidechain: Option<&[f32]>) {
        if !left.is_empty() {
            left[0] = f32::NAN;
        }
        if !right.is_empty() {
            right[0] = f32::INFINITY;
        }
    }

    fn reset(&mut self) {}

    fn parameters(&self) -> &[PluginParameter] {
        &[]
    }
}

#[test]
fn test_crashing_plugin_panic_interception() {
    let mock = Box::new(CrashingMockPlugin::new());
    let mut slot = HardenedPluginSlot::new(0, mock);

    let mut left = vec![0.5; 64];
    let mut right = vec![0.5; 64];

    // Processing must NOT crash the host thread
    let result = slot.safe_process(&mut left, &mut right, None);
    assert!(result.is_err());

    // Slot must transition to Faulted state
    match &slot.status {
        PluginStatus::Faulted(msg) => {
            assert!(msg.contains("FATAL: Segmentation fault"));
        }
        _ => panic!("Expected slot to be Faulted"),
    }

    // Subsequent process calls must safely bypass without panicking
    let subsequent_result = slot.safe_process(&mut left, &mut right, None);
    assert!(subsequent_result.is_ok());
}

#[test]
fn test_rogue_nan_generating_plugin_isolation() {
    let mock = Box::new(RogueNanMockPlugin::new());
    let mut slot = HardenedPluginSlot::new(1, mock);

    let mut left = vec![0.5; 64];
    let mut right = vec![0.5; 64];

    let result = slot.safe_process(&mut left, &mut right, None);
    assert!(result.is_err());

    // Output buffers must be zeroed to prevent loudspeaker / ear damage
    assert_eq!(left[0], 0.0);
    assert_eq!(right[0], 0.0);

    match &slot.status {
        PluginStatus::Faulted(msg) => {
            assert!(msg.contains("NaN/Inf"));
        }
        _ => panic!("Expected slot to be Faulted"),
    }
}
