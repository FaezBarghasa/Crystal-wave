use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct PluginParameter {
    pub id: u32,
    pub name: String,
    pub min_value: f32,
    pub max_value: f32,
    pub default_value: f32,
    pub unit: String,
    raw_bits: Arc<AtomicU32>,
}

impl PluginParameter {
    pub fn new(
        id: u32,
        name: impl Into<String>,
        min_value: f32,
        max_value: f32,
        default_value: f32,
        unit: impl Into<String>,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            min_value,
            max_value,
            default_value,
            unit: unit.into(),
            raw_bits: Arc::new(AtomicU32::new(default_value.to_bits())),
        }
    }

    pub fn get_value(&self) -> f32 {
        f32::from_bits(self.raw_bits.load(Ordering::Relaxed))
    }

    pub fn set_value(&self, val: f32) {
        let clamped = val.clamp(self.min_value, self.max_value);
        self.raw_bits.store(clamped.to_bits(), Ordering::Relaxed);
    }

    pub fn normalized_value(&self) -> f32 {
        let range = self.max_value - self.min_value;
        if range.abs() < 1e-6 {
            0.0
        } else {
            ((self.get_value() - self.min_value) / range).clamp(0.0, 1.0)
        }
    }

    pub fn set_normalized_value(&self, norm: f32) {
        let range = self.max_value - self.min_value;
        let actual = self.min_value + norm.clamp(0.0, 1.0) * range;
        self.set_value(actual);
    }
}
