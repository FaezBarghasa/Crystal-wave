use crate::plugins::plugin_descriptor::PluginDescriptor;
use crate::plugins::plugin_host::{
    AudioPlugin, NativeChannelEq, NativeChromaVerb, NativeCompressor, NativeTapeDelay,
};
use std::collections::HashMap;
use std::sync::Arc;

pub type PluginFactory = Arc<dyn Fn(f32) -> Box<dyn AudioPlugin> + Send + Sync>;

#[derive(Clone)]
pub struct RegisteredPlugin {
    pub descriptor: PluginDescriptor,
    pub is_builtin: bool,
    pub is_enabled: bool,
    pub factory: PluginFactory,
}

/// Central registry managing all built-in and external plugins.
/// Allows the user to dynamically list, enable, disable, and permanently remove any plugin.
pub struct PluginRegistry {
    plugins: HashMap<u32, RegisteredPlugin>,
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            plugins: HashMap::new(),
        };

        // 1. Native Tape Delay (1001)
        registry.register(
            NativeTapeDelay::new(44100.0).descriptor().clone(),
            true,
            |sr| Box::new(NativeTapeDelay::new(sr)),
        );

        // 2. Native Studio Compressor (1002)
        registry.register(
            NativeCompressor::new(44100.0).descriptor().clone(),
            true,
            |sr| Box::new(NativeCompressor::new(sr)),
        );

        // 3. Native Channel EQ (1003)
        registry.register(
            NativeChannelEq::new(44100.0).descriptor().clone(),
            true,
            |sr| Box::new(NativeChannelEq::new(sr)),
        );

        // 4. Fruity Soft Clipper (1004)
        registry.register(
            crate::fl_studio::soft_clipper::FruitySoftClipper::default()
                .descriptor()
                .clone(),
            true,
            |_sr| Box::new(crate::fl_studio::soft_clipper::FruitySoftClipper::default()),
        );

        // 5. Native ChromaVerb (1005)
        registry.register(
            NativeChromaVerb::new(44100.0).descriptor().clone(),
            true,
            |sr| Box::new(NativeChromaVerb::new(sr)),
        );

        // 6. Gross Beat Time Manipulation Engine (1006)
        registry.register(
            crate::fl_studio::gross_beat::GrossBeatEngine::new(44100.0)
                .descriptor()
                .clone(),
            true,
            |sr| Box::new(crate::fl_studio::gross_beat::GrossBeatEngine::new(sr)),
        );

        // 7. Subtractive Synthesizer (1007)
        registry.register(
            crate::plugins::plugin_host::NativeSubtractiveSynth::new(44100.0)
                .descriptor()
                .clone(),
            true,
            |sr| Box::new(crate::plugins::plugin_host::NativeSubtractiveSynth::new(sr)),
        );

        // 8. Drum Synthesizer (1008)
        registry.register(
            crate::plugins::plugin_host::NativeDrumSynth::new(44100.0)
                .descriptor()
                .clone(),
            true,
            |sr| Box::new(crate::plugins::plugin_host::NativeDrumSynth::new(sr)),
        );

        registry
    }

    /// Register a new plugin (built-in or third-party CLAP/VST3)
    pub fn register(
        &mut self,
        descriptor: PluginDescriptor,
        is_builtin: bool,
        factory: impl Fn(f32) -> Box<dyn AudioPlugin> + Send + Sync + 'static,
    ) -> u32 {
        let id = descriptor.unique_id;
        self.plugins.insert(
            id,
            RegisteredPlugin {
                descriptor,
                is_builtin,
                is_enabled: true,
                factory: Arc::new(factory),
            },
        );
        id
    }

    /// Permanently remove (uninstall/unregister) a plugin from the registry.
    /// User can remove any plugin, including built-ins.
    pub fn remove_plugin(&mut self, unique_id: u32) -> bool {
        self.plugins.remove(&unique_id).is_some()
    }

    /// Disable a plugin without uninstalling it
    pub fn disable_plugin(&mut self, unique_id: u32) -> bool {
        if let Some(entry) = self.plugins.get_mut(&unique_id) {
            entry.is_enabled = false;
            true
        } else {
            false
        }
    }

    /// Enable a previously disabled plugin
    pub fn enable_plugin(&mut self, unique_id: u32) -> bool {
        if let Some(entry) = self.plugins.get_mut(&unique_id) {
            entry.is_enabled = true;
            true
        } else {
            false
        }
    }

    pub fn is_enabled(&self, unique_id: u32) -> bool {
        self.plugins
            .get(&unique_id)
            .map(|p| p.is_enabled)
            .unwrap_or(false)
    }

    pub fn get_descriptor(&self, unique_id: u32) -> Option<&PluginDescriptor> {
        self.plugins.get(&unique_id).map(|p| &p.descriptor)
    }

    pub fn list_all(&self) -> Vec<&PluginDescriptor> {
        let mut list: Vec<&PluginDescriptor> =
            self.plugins.values().map(|p| &p.descriptor).collect();
        list.sort_by_key(|d| d.unique_id);
        list
    }

    pub fn list_entries(&self) -> Vec<&RegisteredPlugin> {
        let mut list: Vec<&RegisteredPlugin> = self.plugins.values().collect();
        list.sort_by_key(|p| p.descriptor.unique_id);
        list
    }

    pub fn list_enabled(&self) -> Vec<&PluginDescriptor> {
        let mut list: Vec<&PluginDescriptor> = self
            .plugins
            .values()
            .filter(|p| p.is_enabled)
            .map(|p| &p.descriptor)
            .collect();
        list.sort_by_key(|d| d.unique_id);
        list
    }

    pub fn len(&self) -> usize {
        self.plugins.len()
    }

    pub fn is_empty(&self) -> bool {
        self.plugins.is_empty()
    }

    /// Instantiate an enabled plugin for an audio track / insert slot
    pub fn instantiate(&self, unique_id: u32, sample_rate: f32) -> Option<Box<dyn AudioPlugin>> {
        let entry = self.plugins.get(&unique_id)?;
        if !entry.is_enabled {
            return None;
        }
        Some((entry.factory)(sample_rate))
    }
}
