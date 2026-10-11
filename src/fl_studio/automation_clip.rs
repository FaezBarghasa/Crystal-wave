#[derive(Debug, Clone, PartialEq)]
pub struct SplineNode {
    pub tick: u64,
    pub value: f32,   // Normalized 0.0 to 1.0
    pub tension: f32, // -1.0 to +1.0 (FL Studio curve handle: 0.0 = linear, >0 convex, <0 concave)
}

impl SplineNode {
    pub fn new(tick: u64, value: f32, tension: f32) -> Self {
        Self {
            tick,
            value: value.clamp(0.0, 1.0),
            tension: tension.clamp(-1.0, 1.0),
        }
    }
}

/// Standalone Untethered FL Studio-style Automation Clip
#[derive(Debug, Clone)]
pub struct AutomationClip {
    pub id: usize,
    pub name: String,
    pub nodes: Vec<SplineNode>,
    pub target_param_id: Option<u32>,
}

impl AutomationClip {
    pub fn new(id: usize, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            nodes: vec![
                SplineNode::new(0, 0.0, 0.0),
                SplineNode::new(3840 * 4, 1.0, 0.0),
            ],
            target_param_id: None,
        }
    }

    pub fn add_node(&mut self, tick: u64, value: f32, tension: f32) {
        self.nodes.push(SplineNode::new(tick, value, tension));
        self.nodes.sort_by_key(|n| n.tick);
    }

    /// Evaluates the automation curve at tick `t` with tension curvature
    pub fn evaluate(&self, tick: u64) -> f32 {
        if self.nodes.is_empty() {
            return 0.0;
        }
        if tick <= self.nodes.first().unwrap().tick {
            return self.nodes.first().unwrap().value;
        }
        if tick >= self.nodes.last().unwrap().tick {
            return self.nodes.last().unwrap().value;
        }

        // Find surrounding nodes
        for i in 0..self.nodes.len().saturating_sub(1) {
            let n0 = &self.nodes[i];
            let n1 = &self.nodes[i + 1];

            if tick >= n0.tick && tick <= n1.tick {
                let segment_len = (n1.tick - n0.tick) as f32;
                if segment_len <= 0.0 {
                    return n0.value;
                }
                let mut progress = ((tick - n0.tick) as f32) / segment_len;

                // Apply FL Studio tension curve shaping
                if n0.tension != 0.0 {
                    if n0.tension > 0.0 {
                        progress = progress.powf(1.0 + n0.tension * 3.0);
                    } else {
                        progress = 1.0 - (1.0 - progress).powf(1.0 - n0.tension * 3.0);
                    }
                }

                return n0.value + (n1.value - n0.value) * progress;
            }
        }

        0.0
    }
}
