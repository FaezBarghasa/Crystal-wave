#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AutomationMode {
    #[default]
    Read,
    Touch,
    Latch,
    Write,
    Off,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AutomationScope {
    #[default]
    TrackBased,  // TBA
    RegionBased, // RBA
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AutomationFollowsRegions {
    Never,
    #[default]
    Ask,
    Always,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AutomationPoint {
    pub tick: u64,
    pub value: f32,
    pub curve_tension: f32, // -1.0 to +1.0 Bezier curve bending
}

#[derive(Debug, Clone, PartialEq)]
pub struct AutomationLane {
    pub parameter_id: u32,
    pub name: String,
    pub scope: AutomationScope,
    pub points: Vec<AutomationPoint>,
    pub trim_offset: f32,
}

impl AutomationLane {
    pub fn new(parameter_id: u32, name: impl Into<String>, scope: AutomationScope) -> Self {
        Self {
            parameter_id,
            name: name.into(),
            scope,
            points: Vec::new(),
            trim_offset: 0.0,
        }
    }

    pub fn add_point(&mut self, tick: u64, value: f32, curve_tension: f32) {
        self.points.push(AutomationPoint {
            tick,
            value,
            curve_tension: curve_tension.clamp(-1.0, 1.0),
        });
        self.points.sort_by_key(|p| p.tick);
    }

    /// Interpolate value at target tick with Bezier curve tension
    pub fn interpolate_value(&self, tick: u64) -> f32 {
        if self.points.is_empty() {
            return self.trim_offset;
        }
        if tick <= self.points[0].tick {
            return self.points[0].value + self.trim_offset;
        }
        if tick >= self.points.last().unwrap().tick {
            return self.points.last().unwrap().value + self.trim_offset;
        }

        // Find surrounding points
        for i in 0..self.points.len() - 1 {
            let p0 = &self.points[i];
            let p1 = &self.points[i + 1];
            if tick >= p0.tick && tick <= p1.tick {
                let range = (p1.tick - p0.tick) as f32;
                if range == 0.0 {
                    return p0.value + self.trim_offset;
                }
                let mut t = (tick - p0.tick) as f32 / range;
                // Bezier curve bending
                if p0.curve_tension != 0.0 {
                    let tension = p0.curve_tension;
                    if tension > 0.0 {
                        t = t.powf(1.0 + tension * 2.0);
                    } else {
                        t = 1.0 - (1.0 - t).powf(1.0 - tension * 2.0);
                    }
                }
                let val = p0.value + t * (p1.value - p0.value);
                return val + self.trim_offset;
            }
        }

        self.trim_offset
    }

    /// Automation thinning algorithm (Ramer-Douglas-Peucker reduction)
    pub fn thin_nodes(&mut self, epsilon: f32) {
        if self.points.len() <= 2 {
            return;
        }
        self.points = rdp_thin(&self.points, epsilon);
    }
}

fn rdp_thin(points: &[AutomationPoint], epsilon: f32) -> Vec<AutomationPoint> {
    if points.len() <= 2 {
        return points.to_vec();
    }

    let mut dmax = 0.0f32;
    let mut index = 0usize;
    let end = points.len() - 1;

    for i in 1..end {
        let d = perpendicular_distance(&points[i], &points[0], &points[end]);
        if d > dmax {
            index = i;
            dmax = d;
        }
    }

    if dmax > epsilon {
        let mut rec1 = rdp_thin(&points[..=index], epsilon);
        let rec2 = rdp_thin(&points[index..], epsilon);
        rec1.pop();
        rec1.extend(rec2);
        rec1
    } else {
        vec![points[0].clone(), points[end].clone()]
    }
}

fn perpendicular_distance(pt: &AutomationPoint, line_start: &AutomationPoint, line_end: &AutomationPoint) -> f32 {
    let dx = (line_end.tick as f32) - (line_start.tick as f32);
    let dy = line_end.value - line_start.value;
    let len = (dx * dx + dy * dy).sqrt();
    if len == 0.0 {
        return (pt.value - line_start.value).abs();
    }
    let nom = (dy * (pt.tick as f32) - dx * pt.value + (line_end.tick as f32) * line_start.value - line_end.value * (line_start.tick as f32)).abs();
    nom / len
}

/// Automation Quick Access (AQA) mapper: maps single hardware fader to active parameter
#[derive(Debug, Clone)]
pub struct AutomationQuickAccess {
    pub enabled: bool,
    pub midi_cc_channel: u8,
    pub midi_cc_number: u8,
    pub active_track_id: usize,
    pub active_parameter_id: u32,
}

impl Default for AutomationQuickAccess {
    fn default() -> Self {
        Self {
            enabled: true,
            midi_cc_channel: 1,
            midi_cc_number: 1, // Default CC 1 or learned
            active_track_id: 0,
            active_parameter_id: 0,
        }
    }
}
