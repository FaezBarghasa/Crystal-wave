#[derive(Debug, Clone)]
pub struct ParameterMapping {
    pub target_plugin_slot: usize,
    pub parameter_id: u32,
    pub min_value: f32,
    pub max_value: f32,
    pub invert: bool,
}

#[derive(Debug, Clone)]
pub struct SmartKnob {
    pub label: String,
    pub normalized_value: f32, // 0.0 to 1.0
    pub mappings: Vec<ParameterMapping>,
}

impl SmartKnob {
    pub fn new(label: String) -> Self {
        Self {
            label,
            normalized_value: 0.0,
            mappings: Vec::new(),
        }
    }

    pub fn add_mapping(&mut self, mapping: ParameterMapping) {
        self.mappings.push(mapping);
    }

    pub fn set_value(&mut self, val: f32) {
        self.normalized_value = val.clamp(0.0, 1.0);
    }

    /// Evaluate mapped real-world parameter value
    pub fn evaluate_mapping(&self, mapping_index: usize) -> Option<f32> {
        let mapping = self.mappings.get(mapping_index)?;
        let val = if mapping.invert {
            1.0 - self.normalized_value
        } else {
            self.normalized_value
        };
        Some(mapping.min_value + val * (mapping.max_value - mapping.min_value))
    }
}

pub struct SmartControlsLayout {
    pub layout_name: String,
    pub knobs: Vec<SmartKnob>,
}

impl SmartControlsLayout {
    pub fn new(layout_name: String) -> Self {
        Self {
            layout_name,
            knobs: Vec::new(),
        }
    }
}
