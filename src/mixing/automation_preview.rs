use std::collections::HashMap;

/// Automation "Preview" Mode manager.
/// Allows engineers to audition adjustments to knobs and faders without
/// overwriting existing automation lanes until explicitly written to the timeline.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct AutomationPreviewManager {
    pub is_preview_active: bool,
    /// Staged parameter modifications: `(track_id, param_name) -> staged_audition_value`
    pub staged_parameters: HashMap<(usize, String), f32>,
}

impl AutomationPreviewManager {
    pub fn new() -> Self {
        Self {
            is_preview_active: false,
            staged_parameters: HashMap::new(),
        }
    }

    /// Engages preview audition mode
    pub fn engage_preview(&mut self) {
        self.is_preview_active = true;
    }

    /// Cancels audition mode and reverts all knobs back to active automation
    pub fn cancel_preview(&mut self) {
        self.is_preview_active = false;
        self.staged_parameters.clear();
    }

    /// Auditions a parameter value without committing to automation
    pub fn audition_param(&mut self, track_id: usize, param_name: &str, value: f32) {
        if self.is_preview_active {
            self.staged_parameters
                .insert((track_id, param_name.to_string()), value);
        }
    }

    /// Retrieves either the audition value (if previewing) or the underlying automated value
    pub fn get_effective_value(
        &self,
        track_id: usize,
        param_name: &str,
        current_automated_val: f32,
    ) -> f32 {
        if self.is_preview_active
            && let Some(&audition_val) = self
                .staged_parameters
                .get(&(track_id, param_name.to_string()))
        {
            return audition_val;
        }
        current_automated_val
    }

    /// Writes all auditioned changes to permanent automation points at `current_tick`
    pub fn commit_write_preview(&mut self) -> Vec<(usize, String, f32)> {
        self.is_preview_active = false;
        let mut committed = Vec::new();
        for ((track_id, param_name), val) in self.staged_parameters.drain() {
            committed.push((track_id, param_name, val));
        }
        committed
    }
}
