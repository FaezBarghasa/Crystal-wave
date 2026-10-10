use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct Take {
    pub id: usize,
    pub name: String,
    pub buffer: Arc<Mutex<Vec<f32>>>,
}

impl Take {
    pub fn new(id: usize, name: impl Into<String>, samples: Vec<f32>) -> Self {
        Self {
            id,
            name: name.into(),
            buffer: Arc::new(Mutex::new(samples)),
        }
    }

    pub fn len(&self) -> usize {
        self.buffer.lock().map(|b| b.len()).unwrap_or(0)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
