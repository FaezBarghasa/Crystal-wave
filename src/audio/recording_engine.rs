use crate::audio::take::Take;
use std::sync::{Arc, Mutex};

pub type AudioTrackBuffer = Vec<f32>;

/// Circular buffer for real-time safe audio stream recording
#[derive(Debug, Clone)]
pub struct CircularAudioBuffer {
    buffer: Vec<f32>,
    write_pos: usize,
    capacity: usize,
}

impl CircularAudioBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: vec![0.0; capacity],
            write_pos: 0,
            capacity,
        }
    }

    pub fn write_samples(&mut self, samples: &[f32]) {
        for &s in samples {
            self.buffer[self.write_pos] = s;
            self.write_pos = (self.write_pos + 1) % self.capacity;
        }
    }

    /// Read the latest N samples in chronological order
    pub fn read_latest(&self, n: usize) -> Vec<f32> {
        let count = n.min(self.capacity);
        let mut out = Vec::with_capacity(count);
        let start_pos = (self.write_pos + self.capacity - count) % self.capacity;
        for i in 0..count {
            let idx = (start_pos + i) % self.capacity;
            out.push(self.buffer[idx]);
        }
        out
    }
}

pub struct RecordingEngine {
    pub armed_tracks: Vec<usize>,
    pub track_buffers: Vec<Arc<Mutex<AudioTrackBuffer>>>,
    pub circular_buffers: Vec<CircularAudioBuffer>,
    pub record_producers: Vec<Option<rtrb::Producer<f32>>>,
    pub takes: Vec<Take>,
    pub is_recording: bool,
    pub sample_rate: f32,
    pub auto_punch_enabled: bool,
    pub punch_in_sample: usize,
    pub punch_out_sample: usize,
    pub current_sample_position: usize,
    pub low_latency_mode: bool,
    pub low_latency_threshold_ms: f32, // typically 1.0 to 20.0 ms
}

impl RecordingEngine {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            armed_tracks: Vec::new(),
            track_buffers: Vec::new(),
            circular_buffers: Vec::new(),
            record_producers: Vec::new(),
            takes: Vec::new(),
            is_recording: false,
            sample_rate,
            auto_punch_enabled: false,
            punch_in_sample: 0,
            punch_out_sample: usize::MAX,
            current_sample_position: 0,
            low_latency_mode: false,
            low_latency_threshold_ms: 5.0,
        }
    }

    pub fn arm_track(&mut self, track_id: usize) {
        if !self.armed_tracks.contains(&track_id) {
            self.armed_tracks.push(track_id);
        }
        while self.track_buffers.len() <= track_id {
            self.track_buffers.push(Arc::new(Mutex::new(Vec::new())));
            // 5 seconds circular buffer per track
            let cap = (self.sample_rate * 5.0).round() as usize;
            self.circular_buffers.push(CircularAudioBuffer::new(cap));
        }
    }

    pub fn arm_track_lock_free(&mut self, track_id: usize, capacity: usize) -> rtrb::Consumer<f32> {
        self.arm_track(track_id);
        let (prod, cons) = rtrb::RingBuffer::new(capacity);
        while self.record_producers.len() <= track_id {
            self.record_producers.push(None);
        }
        self.record_producers[track_id] = Some(prod);
        cons
    }

    pub fn disarm_track(&mut self, track_id: usize) {
        self.armed_tracks.retain(|&id| id != track_id);
    }

    pub fn is_armed(&self, track_id: usize) -> bool {
        self.armed_tracks.contains(&track_id)
    }

    pub fn set_low_latency_mode(&mut self, enabled: bool, threshold_ms: f32) {
        self.low_latency_mode = enabled;
        self.low_latency_threshold_ms = threshold_ms.clamp(1.0, 20.0);
    }

    pub fn set_auto_punch(&mut self, punch_in: usize, punch_out: usize, enabled: bool) {
        self.punch_in_sample = punch_in;
        self.punch_out_sample = punch_out.max(punch_in + 1);
        self.auto_punch_enabled = enabled;
    }

    pub fn start_recording(&mut self, start_sample: usize) {
        self.is_recording = true;
        self.current_sample_position = start_sample;
    }

    pub fn stop_recording(&mut self) {
        self.is_recording = false;
    }

    pub fn append_samples_wait_free(&mut self, track_id: usize, samples: &[f32]) {
        if !self.is_armed(track_id) {
            return;
        }

        // Always update circular buffer
        if track_id < self.circular_buffers.len() {
            self.circular_buffers[track_id].write_samples(samples);
        }

        if !self.is_recording {
            return;
        }

        // Check auto-punch bounds
        let chunk_start = self.current_sample_position;
        let chunk_end = chunk_start + samples.len();

        let should_record = if !self.auto_punch_enabled {
            true
        } else {
            chunk_start < self.punch_out_sample && chunk_end > self.punch_in_sample
        };

        if should_record {
            // Lock-free SPSC streaming
            if let Some(Some(prod)) = self.record_producers.get_mut(track_id) {
                for &s in samples {
                    let _ = prod.push(s);
                }
            }

            // Non-blocking try_lock for legacy buffers
            if track_id < self.track_buffers.len()
                && let Ok(mut buf) = self.track_buffers[track_id].try_lock()
            {
                buf.extend_from_slice(samples);
            }
        }

        self.current_sample_position = chunk_end;
    }

    pub fn append_samples(&mut self, track_id: usize, samples: &[f32]) {
        self.append_samples_wait_free(track_id, samples);
    }

    pub fn commit_take(&mut self, track_id: usize, name: impl Into<String>) -> Option<Take> {
        if track_id < self.track_buffers.len()
            && let Ok(mut buf) = self.track_buffers[track_id].lock()
        {
            let samples = std::mem::take(&mut *buf);
            let take = Take::new(self.takes.len(), name, samples);
            self.takes.push(take.clone());
            return Some(take);
        }
        None
    }
}
