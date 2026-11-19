//! Edison Sample Editor & Audio Recorder
//!
//! Clean-room implementation of FL Studio's flagship waveform manipulation,
//! precision audio editor, spectral analyzer, and sample recorder.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum EdisonTool {
    #[default]
    Select,
    Scrub,
    Zoom,
    Slice,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdisonMarker {
    pub sample_index: usize,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct EdisonEditor {
    pub sample_rate: f32,
    pub channels: usize,
    pub samples_l: Vec<f32>,
    pub samples_r: Vec<f32>,
    pub selection_start: usize,
    pub selection_end: usize,
    pub cursor_pos: usize,
    pub zoom: f32,
    pub scroll_offset: f32,
    pub active_tool: EdisonTool,
    pub markers: Vec<EdisonMarker>,
    pub clipboard_l: Vec<f32>,
    pub clipboard_r: Vec<f32>,
    pub is_recording: bool,
    pub snap_to_zero_crossing: bool,
}

impl Default for EdisonEditor {
    fn default() -> Self {
        Self::new(44100.0)
    }
}

impl EdisonEditor {
    pub fn new(sample_rate: f32) -> Self {
        // Generate a 1-second reference sine tone (440Hz A4) with decaying amplitude
        let total_samples = sample_rate as usize;
        let mut samples_l = Vec::with_capacity(total_samples);
        let mut samples_r = Vec::with_capacity(total_samples);

        for i in 0..total_samples {
            let t = i as f32 / sample_rate;
            let env = (-t * 3.0).exp();
            let wave = (2.0 * std::f32::consts::PI * 440.0 * t).sin() * 0.8 * env;
            samples_l.push(wave);
            samples_r.push(wave);
        }

        Self {
            sample_rate,
            channels: 2,
            samples_l,
            samples_r,
            selection_start: 0,
            selection_end: (sample_rate * 0.5) as usize,
            cursor_pos: 0,
            zoom: 1.0,
            scroll_offset: 0.0,
            active_tool: EdisonTool::Select,
            markers: vec![
                EdisonMarker {
                    sample_index: 0,
                    name: "Start".into(),
                },
                EdisonMarker {
                    sample_index: (sample_rate * 0.5) as usize,
                    name: "Decay Trans".into(),
                },
            ],
            clipboard_l: Vec::new(),
            clipboard_r: Vec::new(),
            is_recording: false,
            snap_to_zero_crossing: true,
        }
    }

    pub fn len(&self) -> usize {
        self.samples_l.len()
    }

    pub fn is_empty(&self) -> bool {
        self.samples_l.is_empty()
    }

    pub fn duration_seconds(&self) -> f32 {
        if self.sample_rate > 0.0 {
            self.samples_l.len() as f32 / self.sample_rate
        } else {
            0.0
        }
    }

    pub fn set_selection(&mut self, start: usize, end: usize) {
        let max_len = self.samples_l.len();
        self.selection_start = start.min(max_len);
        self.selection_end = end.min(max_len);
        if self.selection_start > self.selection_end {
            std::mem::swap(&mut self.selection_start, &mut self.selection_end);
        }
    }

    pub fn select_all(&mut self) {
        self.selection_start = 0;
        self.selection_end = self.samples_l.len();
    }

    /// Peak Normalize the active selection (or entire sample if no selection) to 0 dBFS (1.0)
    pub fn normalize(&mut self) {
        let (start, end) = self.active_range();
        if start >= end || end > self.samples_l.len() {
            return;
        }

        let mut peak: f32 = 0.0;
        for i in start..end {
            peak = peak
                .max(self.samples_l[i].abs())
                .max(self.samples_r[i].abs());
        }

        if peak > 1e-6 {
            let gain = 1.0 / peak;
            for i in start..end {
                self.samples_l[i] *= gain;
                self.samples_r[i] *= gain;
            }
        }
    }

    /// Reverse the active selection
    pub fn reverse(&mut self) {
        let (start, end) = self.active_range();
        if start >= end || end > self.samples_l.len() {
            return;
        }

        self.samples_l[start..end].reverse();
        self.samples_r[start..end].reverse();
    }

    /// Linear fade in across the active selection
    pub fn fade_in(&mut self) {
        let (start, end) = self.active_range();
        if end <= start + 1 {
            return;
        }
        let denom = (end - start - 1) as f32;

        for (idx, i) in (start..end).enumerate() {
            let t = idx as f32 / denom;
            self.samples_l[i] *= t;
            self.samples_r[i] *= t;
        }
    }

    /// Linear fade out across the active selection
    pub fn fade_out(&mut self) {
        let (start, end) = self.active_range();
        if end <= start + 1 {
            return;
        }
        let denom = (end - start - 1) as f32;

        for (idx, i) in (start..end).enumerate() {
            let t = 1.0 - (idx as f32 / denom);
            self.samples_l[i] *= t;
            self.samples_r[i] *= t;
        }
    }

    /// Cut selected samples into internal clipboard
    pub fn cut(&mut self) {
        self.copy();
        let (start, end) = self.active_range();
        if start < end && end <= self.samples_l.len() {
            self.samples_l.drain(start..end);
            self.samples_r.drain(start..end);
            self.selection_end = self.selection_start;
        }
    }

    /// Copy selected samples into internal clipboard
    pub fn copy(&mut self) {
        let (start, end) = self.active_range();
        if start < end && end <= self.samples_l.len() {
            self.clipboard_l = self.samples_l[start..end].to_vec();
            self.clipboard_r = self.samples_r[start..end].to_vec();
        }
    }

    /// Paste from internal clipboard at current cursor
    pub fn paste(&mut self) {
        if self.clipboard_l.is_empty() {
            return;
        }
        let pos = self.cursor_pos.min(self.samples_l.len());
        self.samples_l
            .splice(pos..pos, self.clipboard_l.iter().copied());
        self.samples_r
            .splice(pos..pos, self.clipboard_r.iter().copied());
        self.cursor_pos = pos + self.clipboard_l.len();
    }

    /// Range helper
    pub fn active_range(&self) -> (usize, usize) {
        if self.selection_start < self.selection_end {
            (self.selection_start, self.selection_end)
        } else {
            (0, self.samples_l.len())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edison_normalize_and_reverse() {
        let mut edison = EdisonEditor::new(44100.0);
        edison.samples_l = vec![0.1, 0.5, -0.25, 0.0];
        edison.samples_r = vec![0.1, 0.5, -0.25, 0.0];
        edison.select_all();
        edison.normalize();
        assert!((edison.samples_l[1] - 1.0).abs() < 1e-4);

        edison.reverse();
        assert!((edison.samples_l[2] - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_edison_copy_paste() {
        let mut edison = EdisonEditor::new(44100.0);
        edison.samples_l = vec![0.1, 0.2, 0.3, 0.4];
        edison.samples_r = vec![0.1, 0.2, 0.3, 0.4];
        edison.set_selection(1, 3);
        edison.copy();
        assert_eq!(edison.clipboard_l.len(), 2);
        assert_eq!(edison.clipboard_l, vec![0.2, 0.3]);

        edison.cursor_pos = 4;
        edison.paste();
        assert_eq!(edison.samples_l.len(), 6);
        assert_eq!(edison.samples_l[4], 0.2);
        assert_eq!(edison.samples_l[5], 0.3);
    }

    #[test]
    fn test_edison_fade_in_fade_out() {
        let mut edison = EdisonEditor::new(44100.0);
        edison.samples_l = vec![1.0, 1.0, 1.0, 1.0, 1.0];
        edison.samples_r = vec![1.0, 1.0, 1.0, 1.0, 1.0];
        edison.set_selection(0, 5);
        edison.fade_in();
        assert_eq!(edison.samples_l[0], 0.0);
        assert!(edison.samples_l[4] > 0.7);

        edison.fade_out();
        assert_eq!(edison.samples_l[4], 0.0);
    }
}
