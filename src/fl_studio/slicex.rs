//! Slicex Beat & Transient Audio Slicer
//!
//! Emulates FL Studio's Slicex tool: automatic transient detection,
//! region marker slicing, beat division, and MIDI trigger playback.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlicexMarker {
    pub id: usize,
    pub name: String,
    pub start_sample: usize,
    pub end_sample: usize,
    pub trigger_key: u8, // MIDI trigger note (e.g. 60 = C4)
    pub is_active: bool,
}

#[derive(Debug, Clone)]
pub struct SlicexEngine {
    pub sample_name: String,
    pub sample_rate: f32,
    pub total_samples: usize,
    pub markers: Vec<SlicexMarker>,
    pub active_slice_id: Option<usize>,
    pub auto_dump_to_piano_roll: bool,
    pub sensitivity: f32, // Transient detection threshold
}

impl Default for SlicexEngine {
    fn default() -> Self {
        Self::new(44100.0)
    }
}

impl SlicexEngine {
    pub fn new(sample_rate: f32) -> Self {
        let total_samples = (sample_rate * 2.0) as usize; // 2-second drum loop
        let slice_len = total_samples / 8;

        let mut markers = Vec::with_capacity(8);
        for i in 0..8 {
            markers.push(SlicexMarker {
                id: i,
                name: format!("Slice {}", i + 1),
                start_sample: i * slice_len,
                end_sample: (i + 1) * slice_len,
                trigger_key: 60 + i as u8,
                is_active: false,
            });
        }

        Self {
            sample_name: "Amen Break 174BPM.wav".into(),
            sample_rate,
            total_samples,
            markers,
            active_slice_id: None,
            auto_dump_to_piano_roll: true,
            sensitivity: 0.75,
        }
    }

    pub fn trigger_slice(&mut self, slice_id: usize) {
        self.active_slice_id = Some(slice_id);
        for m in &mut self.markers {
            m.is_active = m.id == slice_id;
        }
    }

    pub fn reset_triggers(&mut self) {
        self.active_slice_id = None;
        for m in &mut self.markers {
            m.is_active = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slicex_markers_and_triggers() {
        let mut slicex = SlicexEngine::new(44100.0);
        assert_eq!(slicex.markers.len(), 8);
        assert_eq!(slicex.active_slice_id, None);

        slicex.trigger_slice(2);
        assert_eq!(slicex.active_slice_id, Some(2));
        assert!(slicex.markers[2].is_active);
        assert!(!slicex.markers[1].is_active);

        slicex.reset_triggers();
        assert_eq!(slicex.active_slice_id, None);
        assert!(!slicex.markers[2].is_active);
    }
}
