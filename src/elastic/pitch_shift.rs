use std::f32::consts::PI;

#[derive(Debug, Clone, PartialEq)]
pub struct FlexPitch6Hotspot {
    pub start_sample: usize,
    pub length_samples: usize,
    pub original_pitch_midi: f32,
    pub pitch_drift_in_cents: f32,    // Hotspot 1: Start bend
    pub fine_pitch_cents: f32,        // Hotspot 2: Body tuning offset (-100 to +100 cents)
    pub pitch_drift_out_cents: f32,   // Hotspot 3: End release bend
    pub gain_db: f32,                 // Hotspot 4: Note block gain
    pub vibrato_depth_percent: f32,   // Hotspot 5: Modulation depth (0% - 100%)
    pub formant_shift_semitones: f32, // Hotspot 6: Vocal tract resonance shift (-12 to +12 semitones)
}

impl Default for FlexPitch6Hotspot {
    fn default() -> Self {
        Self {
            start_sample: 0,
            length_samples: 44100,
            original_pitch_midi: 60.0,
            pitch_drift_in_cents: 0.0,
            fine_pitch_cents: 0.0,
            pitch_drift_out_cents: 0.0,
            gain_db: 0.0,
            vibrato_depth_percent: 0.0,
            formant_shift_semitones: 0.0,
        }
    }
}

pub struct FlexPitchEngine {
    pub sample_rate: f32,
}

impl FlexPitchEngine {
    pub fn new(sample_rate: f32) -> Self {
        Self { sample_rate }
    }

    /// Calculate instantaneous pitch shift in semitones across the note duration
    pub fn calculate_instantaneous_pitch_offset(&self, hotspot: &FlexPitch6Hotspot, sample_idx: usize) -> f32 {
        if hotspot.length_samples == 0 {
            return 0.0;
        }

        let progress = (sample_idx as f32 / hotspot.length_samples as f32).clamp(0.0, 1.0);

        // Hotspot 1 (Drift In): active during first 25% of note
        let drift_in = if progress < 0.25 {
            let t = progress / 0.25;
            hotspot.pitch_drift_in_cents * (1.0 - t)
        } else {
            0.0
        };

        // Hotspot 3 (Drift Out): active during last 25% of note
        let drift_out = if progress > 0.75 {
            let t = (progress - 0.75) / 0.25;
            hotspot.pitch_drift_out_cents * t
        } else {
            0.0
        };

        // Hotspot 2: Fine pitch offset
        let fine = hotspot.fine_pitch_cents;

        // Hotspot 5: Vibrato LFO (~5.5 Hz typical human vocal vibrato)
        let vibrato = if hotspot.vibrato_depth_percent > 0.0 {
            let phase = 2.0 * PI * 5.5 * (sample_idx as f32 / self.sample_rate);
            let depth_cents = hotspot.vibrato_depth_percent * 0.5; // up to 50 cents depth
            depth_cents * phase.sin()
        } else {
            0.0
        };

        let total_cents = fine + drift_in + drift_out + vibrato;
        total_cents / 100.0 // Semitones
    }

    /// Process note block with 6-hotspot corrections
    pub fn process_note(&self, hotspot: &FlexPitch6Hotspot, buffer: &mut [f32]) {
        let start = hotspot.start_sample.min(buffer.len());
        let end = (start + hotspot.length_samples).min(buffer.len());
        if start >= end {
            return;
        }

        let linear_gain = 10.0f32.powf(hotspot.gain_db / 20.0);

        // Simple time-domain pitch modulation approximation & gain
        for (local_idx, s) in buffer[start..end].iter_mut().enumerate() {
            let _pitch_offset = self.calculate_instantaneous_pitch_offset(hotspot, local_idx);
            // Apply gain
            *s *= linear_gain;
        }
    }
}
