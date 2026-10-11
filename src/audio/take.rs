use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct Take {
    pub id: usize,
    pub folder_id: usize,
    pub name: String,
    pub buffer: Arc<Mutex<Vec<f32>>>,
    pub muted: bool,
    pub is_active_comp: bool,
}

impl Take {
    pub fn new(id: usize, name: impl Into<String>, samples: Vec<f32>) -> Self {
        Self {
            id,
            folder_id: 0,
            name: name.into(),
            buffer: Arc::new(Mutex::new(samples)),
            muted: false,
            is_active_comp: false,
        }
    }

    pub fn with_folder(mut self, folder_id: usize) -> Self {
        self.folder_id = folder_id;
        self
    }

    pub fn len(&self) -> usize {
        self.buffer.lock().map(|b| b.len()).unwrap_or(0)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
