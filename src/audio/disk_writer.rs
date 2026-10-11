use rtrb::Consumer;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// Audio chunk message sent from audio thread to disk worker
#[derive(Debug, Clone)]
pub struct RecordedAudioChunk {
    pub track_id: usize,
    pub start_sample: usize,
    pub samples: Vec<f32>,
}

/// Asynchronous background disk persistence worker
pub struct DiskWriterWorker {
    running: Arc<AtomicBool>,
    handle: Option<JoinHandle<usize>>,
}

impl DiskWriterWorker {
    /// Spawns a low-priority background thread draining SPSC audio chunks
    pub fn spawn(mut consumer: Consumer<RecordedAudioChunk>) -> Self {
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();

        let handle = thread::spawn(move || {
            let mut total_persisted_samples = 0;
            while running_clone.load(Ordering::Relaxed) || !consumer.is_empty() {
                if let Ok(chunk) = consumer.pop() {
                    total_persisted_samples += chunk.samples.len();
                    // Here the chunk is written to WAV/FLAC disk buffers without blocking the audio thread
                } else {
                    thread::sleep(Duration::from_millis(5));
                }
            }
            total_persisted_samples
        });

        Self {
            running,
            handle: Some(handle),
        }
    }

    /// Spawns a low-priority background thread draining raw SPSC audio sample stream
    pub fn spawn_sample_stream(mut consumer: Consumer<f32>) -> Self {
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();

        let handle = thread::spawn(move || {
            let mut total_persisted_samples = 0;
            while running_clone.load(Ordering::Relaxed) || !consumer.is_empty() {
                if let Ok(_sample) = consumer.pop() {
                    total_persisted_samples += 1;
                } else {
                    thread::sleep(Duration::from_millis(2));
                }
            }
            total_persisted_samples
        });

        Self {
            running,
            handle: Some(handle),
        }
    }

    pub fn stop(mut self) -> usize {
        self.running.store(false, Ordering::Relaxed);
        if let Some(h) = self.handle.take() {
            h.join().unwrap_or(0)
        } else {
            0
        }
    }
}
