#[derive(Debug, Clone, PartialEq)]
pub struct AudioRegion {
    pub buffer: Vec<f32>,
    pub sample_rate: f32,
    pub muted: bool,
    pub soloed: bool,
    pub gain: f32,
    pub pan: f32,
    pub loop_range: Option<(usize, usize)>,
}

impl AudioRegion {
    pub fn new(buffer: Vec<f32>, sample_rate: f32) -> Self {
        Self {
            buffer,
            sample_rate,
            muted: false,
            soloed: false,
            gain: 1.0,
            pan: 0.0,
            loop_range: None,
        }
    }

    pub fn slice(&self, start: usize, end: usize) -> Option<AudioRegion> {
        if start >= end || end > self.buffer.len() {
            return None;
        }
        Some(AudioRegion::new(self.buffer[start..end].to_vec(), self.sample_rate))
    }

    pub fn split(&self, split_point: usize) -> Option<(AudioRegion, AudioRegion)> {
        if split_point == 0 || split_point >= self.buffer.len() {
            return None;
        }
        let r1 = AudioRegion::new(self.buffer[..split_point].to_vec(), self.sample_rate);
        let r2 = AudioRegion::new(self.buffer[split_point..].to_vec(), self.sample_rate);
        Some((r1, r2))
    }

    pub fn merge(&mut self, other: &AudioRegion) {
        self.buffer.extend_from_slice(&other.buffer);
    }

    pub fn set_loop(&mut self, start: usize, end: usize) {
        self.loop_range = Some((start.min(self.buffer.len()), end.min(self.buffer.len())));
    }

    pub fn crossfade(&mut self, crossfade_len: usize) {
        let len = self.buffer.len();
        let xfade = crossfade_len.min(len / 2);
        for i in 0..xfade {
            let phase = i as f32 / xfade as f32;
            let gain = (phase * std::f32::consts::FRAC_PI_2).sin();
            self.buffer[i] *= gain;
            let end_idx = len - 1 - i;
            self.buffer[end_idx] *= gain;
        }
    }

    pub fn reverse(&mut self) {
        self.buffer.reverse();
    }

    pub fn fade_in(&mut self, samples: usize) {
        let n = samples.min(self.buffer.len());
        for i in 0..n {
            let phase = i as f32 / n as f32;
            self.buffer[i] *= phase;
        }
    }

    pub fn fade_out(&mut self, samples: usize) {
        let n = samples.min(self.buffer.len());
        let len = self.buffer.len();
        for i in 0..n {
            let phase = (n - i) as f32 / n as f32;
            self.buffer[len - n + i] *= phase;
        }
    }

    pub fn bounce_to_buffer(&self) -> Vec<f32> {
        let mut out = self.buffer.clone();
        for s in out.iter_mut() {
            *s *= self.gain;
        }
        out
    }

    pub fn delete(&mut self) {
        self.buffer.clear();
    }

    pub fn duplicate(&self) -> AudioRegion {
        self.clone()
    }

    pub fn mute(&mut self, mute: bool) {
        self.muted = mute;
    }

    pub fn solo(&mut self, solo: bool) {
        self.soloed = solo;
    }

    pub fn set_gain(&mut self, gain: f32) {
        self.gain = gain;
    }

    pub fn set_gain_db(&mut self, gain_db: f32) {
        self.gain = 10.0f32.powf(gain_db / 20.0);
    }

    pub fn set_pan(&mut self, pan: f32) {
        self.pan = pan.clamp(-1.0, 1.0);
    }

    pub fn stop(&mut self) {
        self.soloed = false;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StripSilenceConfig {
    pub threshold_db: f32,       // e.g. -40.0 dB
    pub min_section_length_ms: f32, // e.g. 50.0 ms
    pub pre_attack_ms: f32,      // e.g. 10.0 ms
    pub post_release_ms: f32,    // e.g. 20.0 ms
}

impl Default for StripSilenceConfig {
    fn default() -> Self {
        Self {
            threshold_db: -40.0,
            min_section_length_ms: 50.0,
            pre_attack_ms: 10.0,
            post_release_ms: 20.0,
        }
    }
}

pub struct AudioFileEditor {
    pub buffer: Vec<f32>,
    pub sample_offset: usize,
    pub sample_rate: f32,
}

impl AudioFileEditor {
    pub fn new(buffer: Vec<f32>, sample_rate: f32) -> Self {
        Self {
            buffer,
            sample_offset: 0,
            sample_rate,
        }
    }

    pub fn remove_dc_offset(&mut self) {
        if self.buffer.is_empty() { return; }
        let sum: f32 = self.buffer.iter().sum();
        let dc = sum / self.buffer.len() as f32;
        for s in &mut self.buffer {
            *s -= dc;
        }
    }

    pub fn find_zero_crossing(&self, index: usize) -> usize {
        if index >= self.buffer.len() {
            return self.buffer.len().saturating_sub(1);
        }
        let mut best_idx = index;
        let mut min_val = self.buffer[index].abs();
        let range = 64;
        let start = index.saturating_sub(range);
        let end = (index + range).min(self.buffer.len());
        for i in start..end {
            let val = self.buffer[i].abs();
            if val < min_val {
                min_val = val;
                best_idx = i;
            }
        }
        best_idx
    }

    pub fn reverse(&mut self) {
        self.buffer.reverse();
    }

    pub fn apply_gain_db(&mut self, gain_db: f32) {
        let linear = 10.0f32.powf(gain_db / 20.0);
        for s in &mut self.buffer {
            *s *= linear;
        }
    }

    /// Strip silence and return non-silent sample slices `(start_sample, end_sample)`
    pub fn strip_silence(&self, config: &StripSilenceConfig) -> Vec<(usize, usize)> {
        if self.buffer.is_empty() {
            return Vec::new();
        }

        let threshold_amp = 10.0f32.powf(config.threshold_db / 20.0);
        let pre_samples = ((config.pre_attack_ms / 1000.0) * self.sample_rate).round() as usize;
        let post_samples = ((config.post_release_ms / 1000.0) * self.sample_rate).round() as usize;
        let min_samples = ((config.min_section_length_ms / 1000.0) * self.sample_rate).round() as usize;

        let mut in_active = false;
        let mut current_start = 0;
        let mut raw_ranges = Vec::new();

        for (i, &s) in self.buffer.iter().enumerate() {
            let active = s.abs() >= threshold_amp;
            if active && !in_active {
                in_active = true;
                current_start = i;
            } else if !active && in_active {
                in_active = false;
                raw_ranges.push((current_start, i));
            }
        }
        if in_active {
            raw_ranges.push((current_start, self.buffer.len()));
        }

        // Apply pre-attack and post-release and merge overlapping ranges
        let mut expanded_ranges: Vec<(usize, usize)> = Vec::new();
        for (st, en) in raw_ranges {
            let exp_start = st.saturating_sub(pre_samples);
            let exp_end = (en + post_samples).min(self.buffer.len());

            if let Some(last) = expanded_ranges.last_mut()
                && exp_start <= last.1 {
                    last.1 = last.1.max(exp_end);
                    continue;
                }
            expanded_ranges.push((exp_start, exp_end));
        }

        // Filter by minimum section length
        expanded_ranges
            .into_iter()
            .filter(|(s, e)| (e - s) >= min_samples)
            .collect()
    }
}
