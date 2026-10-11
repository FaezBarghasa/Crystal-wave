pub const MAX_HISTORY_STEPS: usize = 500;

#[derive(Debug, Clone)]
pub struct HistoryAction<T: Clone> {
    pub step_index: usize,
    pub description: String,
    pub timestamp_epoch_ms: u64,
    pub snapshot: T,
}

/// FL Studio 500-Action Visual History & Non-Linear Rollback Engine
pub struct FlHistoryManager<T: Clone> {
    actions: Vec<HistoryAction<T>>,
    current_index: usize,
}

impl<T: Clone> Default for FlHistoryManager<T> {
    fn default() -> Self {
        Self {
            actions: Vec::new(),
            current_index: 0,
        }
    }
}

impl<T: Clone> FlHistoryManager<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_action(&mut self, description: impl Into<String>, snapshot: T) {
        if self.current_index < self.actions.len() {
            self.actions.truncate(self.current_index);
        }

        if self.actions.len() >= MAX_HISTORY_STEPS {
            self.actions.remove(0);
        }

        let step_index = self.actions.len();
        self.actions.push(HistoryAction {
            step_index,
            description: description.into(),
            timestamp_epoch_ms: 0,
            snapshot,
        });

        self.current_index = self.actions.len();
    }

    /// Jump directly to action at index N (e.g. from 3 hours ago)
    pub fn jump_to(&mut self, index: usize) -> Option<&T> {
        if index < self.actions.len() {
            self.current_index = index + 1;
            Some(&self.actions[index].snapshot)
        } else {
            None
        }
    }

    pub fn list_history(&self) -> &[HistoryAction<T>] {
        &self.actions
    }

    pub fn current_action(&self) -> Option<&HistoryAction<T>> {
        if self.current_index > 0 && self.current_index <= self.actions.len() {
            Some(&self.actions[self.current_index - 1])
        } else {
            None
        }
    }
}
