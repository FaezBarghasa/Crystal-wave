use Crystal_wave::plugins::plugin_descriptor::{PluginCategory, PluginDescriptor};
use Crystal_wave::plugins::plugin_parameters::PluginParameter;
use Crystal_wave::plugins::{AudioPlugin, PluginRegistry};

#[test]
fn test_default_builtin_plugins_registration() {
    let registry = PluginRegistry::new();
    let plugins = registry.list_all();

    assert!(plugins.len() >= 5);
    assert!(registry.get_descriptor(1001).is_some()); // Tape Delay
    assert!(registry.get_descriptor(1002).is_some()); // Studio Compressor
    assert!(registry.get_descriptor(1003).is_some()); // Channel EQ
    assert!(registry.get_descriptor(1004).is_some()); // Soft Clipper
    assert!(registry.get_descriptor(1005).is_some()); // Chroma Verb
}

#[test]
fn test_plugin_enable_disable_lifecycle() {
    let mut registry = PluginRegistry::new();

    // Builtin compressor (1002)
    assert!(registry.is_enabled(1002));

    // Should create instance when enabled
    let inst = registry.instantiate(1002, 48000.0);
    assert!(inst.is_some());

    // Disable
    assert!(registry.disable_plugin(1002));
    assert!(!registry.is_enabled(1002));

    // Should not create instance when disabled
    let inst_disabled = registry.instantiate(1002, 48000.0);
    assert!(inst_disabled.is_none());

    // Re-enable
    assert!(registry.enable_plugin(1002));
    assert!(registry.is_enabled(1002));
    assert!(registry.instantiate(1002, 48000.0).is_some());
}

#[test]
fn test_plugin_permanent_removal() {
    let mut registry = PluginRegistry::new();
    let initial_count = registry.len();

    // User permanently removes Chroma Verb (1005)
    assert!(registry.remove_plugin(1005));
    assert_eq!(registry.len(), initial_count - 1);
    assert!(registry.get_descriptor(1005).is_none());
    assert!(registry.instantiate(1005, 48000.0).is_none());

    // Removing again returns false
    assert!(!registry.remove_plugin(1005));
}

#[test]
fn test_custom_plugin_registration_and_dsp_execution() {
    let mut registry = PluginRegistry::new();

    struct SimpleGainPlugin {
        desc: PluginDescriptor,
        gain: f32,
    }

    impl AudioPlugin for SimpleGainPlugin {
        fn descriptor(&self) -> &PluginDescriptor {
            &self.desc
        }
        fn process(&mut self, left: &mut [f32], right: &mut [f32], _sidechain: Option<&[f32]>) {
            for sample in left.iter_mut() {
                *sample *= self.gain;
            }
            for sample in right.iter_mut() {
                *sample *= self.gain;
            }
        }
        fn reset(&mut self) {}
        fn parameters(&self) -> &[PluginParameter] {
            &[]
        }
    }

    let custom_desc = PluginDescriptor::new(9999, "Custom Precision Gain", PluginCategory::Utility);
    let desc_clone = custom_desc.clone();

    registry.register(custom_desc, false, move |_sr| {
        Box::new(SimpleGainPlugin {
            desc: desc_clone.clone(),
            gain: 2.0,
        })
    });

    let mut instance = registry
        .instantiate(9999, 44100.0)
        .expect("Instantiated custom");
    let mut left_audio = [0.25_f32, -0.5_f32, 0.1_f32];
    let mut right_audio = [0.1_f32, -0.2_f32, 0.3_f32];
    instance.process(&mut left_audio, &mut right_audio, None);

    assert_eq!(left_audio, [0.5_f32, -1.0_f32, 0.2_f32]);
    assert_eq!(right_audio, [0.2_f32, -0.4_f32, 0.6_f32]);

    // Uninstall custom plugin
    assert!(registry.remove_plugin(9999));
    assert!(registry.instantiate(9999, 44100.0).is_none());
}
