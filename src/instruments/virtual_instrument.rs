use crate::instruments::instrument::{VirtualInstrument, VoiceStealingMode};
use std::f32::consts::PI;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OscillatorWaveform {
    Sine,
    #[default]
    Sawtooth,
    Square,
    Triangle,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AdsrEnvelope {
    pub attack_samples: usize,
    pub decay_samples: usize,
    pub sustain_level: f32, // 0.0 to 1.0
    pub release_samples: usize,
}

impl Default for AdsrEnvelope {
    fn default() -> Self {
        Self {
            attack_samples: 441, // 10ms at 44.1k
            decay_samples: 4410, // 100ms
            sustain_level: 0.7,
            release_samples: 8820, // 200ms
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SynthVoiceState {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

#[derive(Debug, Clone)]
pub struct SynthVoice {
    pub note: u8,
    pub velocity: f32,
    pub phase: f32,
    pub state: SynthVoiceState,
    pub env_stage_sample: usize,
    pub current_env_level: f32,
    pub age: u64,
}

impl Default for SynthVoice {
    fn default() -> Self {
        Self::new()
    }
}

impl SynthVoice {
    pub fn new() -> Self {
        Self {
            note: 0,
            velocity: 0.0,
            phase: 0.0,
            state: SynthVoiceState::Idle,
            env_stage_sample: 0,
            current_env_level: 0.0,
            age: 0,
        }
    }

    pub fn trigger(&mut self, note: u8, vel: f32) {
        self.note = note;
        self.velocity = vel;
        self.phase = 0.0;
        self.state = SynthVoiceState::Attack;
        self.env_stage_sample = 0;
        self.current_env_level = 0.0;
        self.age = 0;
    }

    pub fn release(&mut self) {
        if self.state != SynthVoiceState::Idle {
            self.state = SynthVoiceState::Release;
            self.env_stage_sample = 0;
        }
    }

    pub fn is_active(&self) -> bool {
        self.state != SynthVoiceState::Idle
    }
}

pub struct PolyphonicSynth {
    pub name: String,
    pub waveform: OscillatorWaveform,
    pub adsr: AdsrEnvelope,
    pub voices: Vec<SynthVoice>,
    pub max_voices: usize,
    pub stealing_mode: VoiceStealingMode,
    pub sample_rate: f32,
    pub pitch_bend_semitones: f32,
    pub filter_cutoff_hz: f32,
    global_age_counter: u64,
}

impl PolyphonicSynth {
    pub fn new(name: impl Into<String>, max_voices: usize, sample_rate: f32) -> Self {
        let max_v = max_voices.clamp(1, 64);
        let mut voices = Vec::with_capacity(max_v);
        for _ in 0..max_v {
            voices.push(SynthVoice::new());
        }

        Self {
            name: name.into(),
            waveform: OscillatorWaveform::Sawtooth,
            adsr: AdsrEnvelope::default(),
            voices,
            max_voices: max_v,
            stealing_mode: VoiceStealingMode::Oldest,
            sample_rate,
            pitch_bend_semitones: 0.0,
            filter_cutoff_hz: 5000.0,
            global_age_counter: 0,
        }
    }

    pub fn midi_to_freq(note: u8, bend_semitones: f32) -> f32 {
        let semitones = note as f32 + bend_semitones;
        440.0 * 2.0f32.powf((semitones - 69.0) / 12.0)
    }
}

impl VirtualInstrument for PolyphonicSynth {
    fn name(&self) -> &str {
        &self.name
    }

    fn note_on(&mut self, note: u8, velocity: u16) {
        let vel_norm = (velocity as f32 / 65535.0).clamp(0.01, 1.0);
        self.global_age_counter += 1;

        // Try to find idle voice
        if let Some(v) = self.voices.iter_mut().find(|v| !v.is_active()) {
            v.trigger(note, vel_norm);
            v.age = self.global_age_counter;
            return;
        }

        // Steal voice if all active
        let mut best_idx = 0;
        let mut oldest_age = u64::MAX;
        for (i, v) in self.voices.iter().enumerate() {
            if v.age < oldest_age {
                oldest_age = v.age;
                best_idx = i;
            }
        }

        self.voices[best_idx].trigger(note, vel_norm);
        self.voices[best_idx].age = self.global_age_counter;
    }

    fn note_off(&mut self, note: u8) {
        for v in &mut self.voices {
            if v.is_active() && v.note == note {
                v.release();
            }
        }
    }

    fn pitch_bend(&mut self, semitones: f32) {
        self.pitch_bend_semitones = semitones;
    }

    fn aftertouch(&mut self, _pressure: u16) {}

    fn render_block(&mut self, left: &mut [f32], right: &mut [f32]) {
        let len = left.len().min(right.len());
        let adsr = self.adsr;
        let sr = self.sample_rate;
        let bend = self.pitch_bend_semitones;
        let wave = self.waveform;

        for voice in &mut self.voices {
            if !voice.is_active() {
                continue;
            }

            let freq = Self::midi_to_freq(voice.note, bend);
            let phase_inc = (2.0 * PI * freq) / sr;

            for i in 0..len {
                // Envelope computation
                match voice.state {
                    SynthVoiceState::Idle => {
                        voice.current_env_level = 0.0;
                    }
                    SynthVoiceState::Attack => {
                        voice.env_stage_sample += 1;
                        let progress =
                            voice.env_stage_sample as f32 / adsr.attack_samples.max(1) as f32;
                        voice.current_env_level = progress.clamp(0.0, 1.0);
                        if voice.env_stage_sample >= adsr.attack_samples {
                            voice.state = SynthVoiceState::Decay;
                            voice.env_stage_sample = 0;
                        }
                    }
                    SynthVoiceState::Decay => {
                        voice.env_stage_sample += 1;
                        let progress =
                            voice.env_stage_sample as f32 / adsr.decay_samples.max(1) as f32;
                        voice.current_env_level = (1.0 - progress * (1.0 - adsr.sustain_level))
                            .clamp(adsr.sustain_level, 1.0);
                        if voice.env_stage_sample >= adsr.decay_samples {
                            voice.state = SynthVoiceState::Sustain;
                        }
                    }
                    SynthVoiceState::Sustain => {
                        voice.current_env_level = adsr.sustain_level;
                    }
                    SynthVoiceState::Release => {
                        voice.env_stage_sample += 1;
                        let progress =
                            voice.env_stage_sample as f32 / adsr.release_samples.max(1) as f32;
                        voice.current_env_level = (adsr.sustain_level * (1.0 - progress)).max(0.0);
                        if voice.env_stage_sample >= adsr.release_samples {
                            voice.state = SynthVoiceState::Idle;
                            break;
                        }
                    }
                }

                // Oscillator sample
                let osc_sample = match wave {
                    OscillatorWaveform::Sine => voice.phase.sin(),
                    OscillatorWaveform::Sawtooth => (voice.phase / PI) - 1.0,
                    OscillatorWaveform::Square => {
                        if voice.phase < PI {
                            1.0
                        } else {
                            -1.0
                        }
                    }
                    OscillatorWaveform::Triangle => (voice.phase / PI - 1.0).abs() * 2.0 - 1.0,
                };

                let sample = osc_sample * voice.current_env_level * voice.velocity * 0.25;
                left[i] += sample;
                right[i] += sample;

                voice.phase = (voice.phase + phase_inc) % (2.0 * PI);
            }
        }
    }

    fn active_voices(&self) -> usize {
        self.voices.iter().filter(|v| v.is_active()).count()
    }

    fn reset(&mut self) {
        for v in &mut self.voices {
            v.state = SynthVoiceState::Idle;
        }
    }
}
