#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutomationShape {
    Linear,
    Spline,
    SCurve,
    LfoSine,
    LfoSaw,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AutomationNode {
    pub tick_offset: u64,
    pub value: f32,   // 0.0 to 1.0 normalized
    pub tension: f32, // -1.0 to 1.0 curvature
}

/// A free-form floating automation object independent of any fixed track or region.
/// Can be dragged, dropped, or stretched across any track or parameter.
#[derive(Debug, Clone, PartialEq)]
pub struct FloatingAutomationObject {
    pub id: usize,
    pub name: String,
    pub length_ticks: u64,
    pub shape: AutomationShape,
    pub nodes: Vec<AutomationNode>,
    pub assigned_track_id: Option<usize>,
    pub assigned_param_name: Option<String>,
}

impl FloatingAutomationObject {
    pub fn new(id: usize, name: &str, length_ticks: u64, shape: AutomationShape) -> Self {
        let mut obj = Self {
            id,
            name: name.to_string(),
            length_ticks: length_ticks.max(1),
            shape,
            nodes: Vec::new(),
            assigned_track_id: None,
            assigned_param_name: None,
        };

        // Add default start and end nodes
        obj.nodes.push(AutomationNode {
            tick_offset: 0,
            value: 0.0,
            tension: 0.0,
        });
        obj.nodes.push(AutomationNode {
            tick_offset: length_ticks,
            value: 1.0,
            tension: 0.0,
        });

        obj
    }

    pub fn add_node(&mut self, tick_offset: u64, value: f32, tension: f32) {
        let node = AutomationNode {
            tick_offset: tick_offset.min(self.length_ticks),
            value: value.clamp(0.0, 1.0),
            tension: tension.clamp(-1.0, 1.0),
        };
        self.nodes.push(node);
        self.nodes.sort_by_key(|n| n.tick_offset);
    }

    /// Evaluates the automation curve at a specific relative tick offset.
    pub fn evaluate(&self, tick_offset: u64) -> f32 {
        if self.nodes.is_empty() {
            return 0.0;
        }

        let clamped_tick = tick_offset.min(self.length_ticks);

        // Find surrounding nodes
        let mut prev = &self.nodes[0];
        let mut next = &self.nodes[self.nodes.len() - 1];

        for i in 0..self.nodes.len() - 1 {
            if clamped_tick >= self.nodes[i].tick_offset && clamped_tick <= self.nodes[i + 1].tick_offset {
                prev = &self.nodes[i];
                next = &self.nodes[i + 1];
                break;
            }
        }

        if prev.tick_offset == next.tick_offset {
            return prev.value;
        }

        let segment_len = (next.tick_offset - prev.tick_offset) as f32;
        let t = ((clamped_tick - prev.tick_offset) as f32) / segment_len;

        match self.shape {
            AutomationShape::Linear => prev.value + t * (next.value - prev.value),
            AutomationShape::Spline | AutomationShape::SCurve => {
                // Apply tension power curve
                let warped_t = if prev.tension >= 0.0 {
                    t.powf(1.0 + prev.tension * 3.0)
                } else {
                    1.0 - (1.0 - t).powf(1.0 - prev.tension * 3.0)
                };
                prev.value + warped_t * (next.value - prev.value)
            }
            AutomationShape::LfoSine => {
                let cycles = 2.0;
                let sine = (2.0 * std::f32::consts::PI * t * cycles).sin() * 0.5 + 0.5;
                prev.value + sine * (next.value - prev.value)
            }
            AutomationShape::LfoSaw => {
                let saw = (t * 4.0).fract();
                prev.value + saw * (next.value - prev.value)
            }
        }
    }

    /// Rescales all nodes proportionally when stretched across a chorus or arrangement span.
    pub fn stretch_to(&mut self, new_length_ticks: u64) {
        if self.length_ticks == 0 || new_length_ticks == 0 {
            return;
        }
        let scale = (new_length_ticks as f64) / (self.length_ticks as f64);
        for node in &mut self.nodes {
            node.tick_offset = ((node.tick_offset as f64) * scale).round() as u64;
        }
        self.length_ticks = new_length_ticks;
    }

    /// Drags and drops the automation object onto a target track parameter.
    pub fn drop_onto_track(&mut self, track_id: usize, param_name: &str) {
        self.assigned_track_id = Some(track_id);
        self.assigned_param_name = Some(param_name.to_string());
    }
}
