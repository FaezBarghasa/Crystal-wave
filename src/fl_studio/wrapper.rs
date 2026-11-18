//! Fruity Wrapper & Channel Settings
//!
//! Emulates FL Studio's Fruity Wrapper and Channel Settings window:
//! Sampler properties, ADSR Envelopes, Pitch transposition, Time Stretching,
//! Polyphony, and Target Mixer Track assignment.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PolyphonyMode {
    #[default]
    Poly,
    Mono,
    Legato,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum WrapperTab {
    #[default]
    Sampler,
    Envelope,
    Misc,
    Fx,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdsrEnvelope {
    pub attack_ms: f32,
    pub decay_ms: f32,
    pub sustain_level: f32, // 0.0 to 1.0
    pub release_ms: f32,
    pub tension: f32, // -1.0 to +1.0
    pub enabled: bool,
}

impl Default for AdsrEnvelope {
    fn default() -> Self {
        Self {
            attack_ms: 10.0,
            decay_ms: 200.0,
            sustain_level: 0.7,
            release_ms: 300.0,
            tension: 0.0,
            enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FruityWrapperState {
    pub channel_id: usize,
    pub active_tab: WrapperTab,
    pub sample_path: String,
    pub root_note: u8, // 60 = C4
    pub pitch_cents: i32,
    pub pitch_range_semitones: u8,
    pub volume: f32,
    pub pan: f32,
    pub polyphony_mode: PolyphonyMode,
    pub max_voices: u8,
    pub slide_time_ms: f32,
    pub target_mixer_track: usize,
    pub volume_env: AdsrEnvelope,
    pub filter_cutoff: f32,
    pub filter_resonance: f32,
    pub reverse: bool,
    pub normalize: bool,
}

impl Default for FruityWrapperState {
    fn default() -> Self {
        Self::new(0)
    }
}

impl FruityWrapperState {
    pub fn new(channel_id: usize) -> Self {
        Self {
            channel_id,
            active_tab: WrapperTab::Sampler,
            sample_path: "808_Kick.wav".into(),
            root_note: 60,
            pitch_cents: 0,
            pitch_range_semitones: 2,
            volume: 1.0,
            pan: 0.0,
            polyphony_mode: PolyphonyMode::Poly,
            max_voices: 32,
            slide_time_ms: 40.0,
            target_mixer_track: channel_id + 1,
            volume_env: AdsrEnvelope::default(),
            filter_cutoff: 20000.0,
            filter_resonance: 0.0,
            reverse: false,
            normalize: false,
        }
    }
}
