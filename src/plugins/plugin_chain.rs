use crate::plugins::plugin_host::AudioPlugin;

pub const MAX_INSERTS_PER_CHANNEL: usize = 15;

pub struct InsertSlot {
    pub slot_id: usize,
    pub plugin: Box<dyn AudioPlugin>,
    pub bypassed: bool,
    pub mix: f32, // 0.0 to 1.0
}

pub struct PluginChain {
    pub slots: Vec<InsertSlot>,
    dry_scratch_l: Vec<f32>,
    dry_scratch_r: Vec<f32>,
}

impl Default for PluginChain {
    fn default() -> Self {
        Self {
            slots: Vec::new(),
            dry_scratch_l: Vec::with_capacity(1024),
            dry_scratch_r: Vec::with_capacity(1024),
        }
    }
}

impl PluginChain {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_plugin(&mut self, plugin: Box<dyn AudioPlugin>) -> bool {
        if self.slots.len() >= MAX_INSERTS_PER_CHANNEL {
            return false;
        }
        let slot_id = self.slots.len();
        self.slots.push(InsertSlot {
            slot_id,
            plugin,
            bypassed: false,
            mix: 1.0,
        });
        true
    }

    pub fn set_bypassed(&mut self, slot_id: usize, bypassed: bool) {
        if let Some(slot) = self.slots.get_mut(slot_id) {
            slot.bypassed = bypassed;
        }
    }

    pub fn set_mix(&mut self, slot_id: usize, mix: f32) {
        if let Some(slot) = self.slots.get_mut(slot_id) {
            slot.mix = mix.clamp(0.0, 1.0);
        }
    }

    pub fn total_latency_samples(&self) -> usize {
        self.slots
            .iter()
            .filter(|s| !s.bypassed)
            .map(|s| s.plugin.latency_samples())
            .sum()
    }

    pub fn process(&mut self, left: &mut [f32], right: &mut [f32], sidechain: Option<&[f32]>) {
        let len = left.len().min(right.len());
        if len == 0 {
            return;
        }

        if self.dry_scratch_l.len() < len {
            self.dry_scratch_l.resize(len, 0.0);
        }
        if self.dry_scratch_r.len() < len {
            self.dry_scratch_r.resize(len, 0.0);
        }

        for slot in &mut self.slots {
            if slot.bypassed {
                continue;
            }

            if (slot.mix - 1.0).abs() > 1e-4 {
                self.dry_scratch_l[..len].copy_from_slice(&left[..len]);
                self.dry_scratch_r[..len].copy_from_slice(&right[..len]);
            }

            slot.plugin.process(left, right, sidechain);

            if (slot.mix - 1.0).abs() > 1e-4 {
                let mix = slot.mix;
                for i in 0..len {
                    left[i] = self.dry_scratch_l[i] * (1.0 - mix) + left[i] * mix;
                    right[i] = self.dry_scratch_r[i] * (1.0 - mix) + right[i] * mix;
                }
            }
        }
    }
}
