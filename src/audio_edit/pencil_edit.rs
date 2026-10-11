#[derive(Debug, Clone, PartialEq)]
pub struct PencilEditPoint {
    pub position: usize,
    pub sample: f32,
}

#[derive(Debug, Clone, Default)]
pub struct PencilEdit {
    pub points: Vec<PencilEditPoint>,
}

impl PencilEdit {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_point(&mut self, position: usize, sample: f32) {
        self.points.push(PencilEditPoint { position, sample: sample.clamp(-1.0, 1.0) });
        self.points.sort_by_key(|p| p.position);
    }

    pub fn apply_to(&self, buffer: &mut [f32]) {
        for pt in &self.points {
            if pt.position < buffer.len() {
                buffer[pt.position] = pt.sample;
            }
        }
    }

    /// Repair pop/click by smooth linear interpolation between clean boundaries
    pub fn repair_pop_click(buffer: &mut [f32], click_start: usize, click_end: usize) {
        if buffer.is_empty() || click_start >= click_end || click_end >= buffer.len() {
            return;
        }

        let left_val = if click_start > 0 { buffer[click_start - 1] } else { 0.0 };
        let right_val = if click_end < buffer.len() { buffer[click_end] } else { 0.0 };
        let span = (click_end - click_start + 1) as f32;

        for (step, i) in (click_start..click_end).enumerate() {
            let t = (step + 1) as f32 / span;
            // Smooth hermite / cosine interpolation
            let smooth_t = (1.0 - (t * std::f32::consts::PI).cos()) * 0.5;
            buffer[i] = left_val * (1.0 - smooth_t) + right_val * smooth_t;
        }
    }
}
