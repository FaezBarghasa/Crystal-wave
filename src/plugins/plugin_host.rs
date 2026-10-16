use crate::plugins::plugin_descriptor::{PluginCategory, PluginDescriptor};
use crate::plugins::plugin_parameters::PluginParameter;

pub trait AudioPlugin: Send + Sync {
    fn descriptor(&self) -> &PluginDescriptor;
    fn process(&mut self, left: &mut [f32], right: &mut [f32], sidechain: Option<&[f32]>);
    fn reset(&mut self);
    fn parameters(&self) -> &[PluginParameter];
    fn latency_samples(&self) -> usize {
        self.descriptor().latency_samples
    }
}

/// Native Logic Pro Tape Delay effect
pub struct NativeTapeDelay {
    descriptor: PluginDescriptor,
    parameters: Vec<PluginParameter>,
    delay_buffer_l: Vec<f32>,
    delay_buffer_r: Vec<f32>,
    write_pos: usize,
    sample_rate: f32,
}

impl Default for NativeTapeDelay {
    fn default() -> Self {
        Self::new(44100.0)
    }
}

impl NativeTapeDelay {
    pub fn new(sample_rate: f32) -> Self {
        let max_samples = (sample_rate * 2.0).round() as usize; // 2 seconds max
        Self {
            descriptor: PluginDescriptor::new(1001, "Tape Delay", PluginCategory::Delay),
            parameters: vec![
                PluginParameter::new(1, "Delay Time (ms)", 1.0, 1000.0, 250.0, "ms"),
                PluginParameter::new(2, "Feedback", 0.0, 0.95, 0.35, "%"),
                PluginParameter::new(3, "Mix", 0.0, 1.0, 0.40, "%"),
            ],
            delay_buffer_l: vec![0.0; max_samples],
            delay_buffer_r: vec![0.0; max_samples],
            write_pos: 0,
            sample_rate,
        }
    }
}

impl AudioPlugin for NativeTapeDelay {
    fn descriptor(&self) -> &PluginDescriptor {
        &self.descriptor
    }

    fn process(&mut self, left: &mut [f32], right: &mut [f32], _sidechain: Option<&[f32]>) {
        let delay_ms = self.parameters[0].get_value();
        let feedback = self.parameters[1].get_value();
        let mix = self.parameters[2].get_value();

        let cap = self.delay_buffer_l.len();
        let delay_samples = ((delay_ms / 1000.0) * self.sample_rate).round() as usize;
        let delay_samples = delay_samples.clamp(1, cap - 1);

        let len = left.len().min(right.len());
        for i in 0..len {
            let read_pos = (self.write_pos + cap - delay_samples) % cap;
            let delayed_l = self.delay_buffer_l[read_pos];
            let delayed_r = self.delay_buffer_r[read_pos];

            self.delay_buffer_l[self.write_pos] = left[i] + delayed_l * feedback;
            self.delay_buffer_r[self.write_pos] = right[i] + delayed_r * feedback;

            left[i] = left[i] * (1.0 - mix) + delayed_l * mix;
            right[i] = right[i] * (1.0 - mix) + delayed_r * mix;

            self.write_pos = (self.write_pos + 1) % cap;
        }
    }

    fn reset(&mut self) {
        self.delay_buffer_l.fill(0.0);
        self.delay_buffer_r.fill(0.0);
        self.write_pos = 0;
    }

    fn parameters(&self) -> &[PluginParameter] {
        &self.parameters
    }
}

/// Native Studio Compressor effect plugin
pub struct NativeCompressor {
    descriptor: PluginDescriptor,
    parameters: Vec<PluginParameter>,
    compressor: crate::dsp::compressor::Compressor,
}

impl NativeCompressor {
    pub fn new(sample_rate: f32) -> Self {
        let config = crate::dsp::compressor::CompressorConfig::default();
        let compressor = crate::dsp::compressor::Compressor::new(config, sample_rate)
            .expect("Valid compressor config");
        Self {
            descriptor: PluginDescriptor::new(1002, "Studio Compressor", PluginCategory::Dynamics),
            parameters: vec![
                PluginParameter::new(1, "Threshold (dB)", -60.0, 0.0, -20.0, "dB"),
                PluginParameter::new(2, "Ratio", 1.0, 20.0, 4.0, ":1"),
                PluginParameter::new(3, "Attack (ms)", 0.1, 200.0, 20.0, "ms"),
                PluginParameter::new(4, "Release (ms)", 1.0, 1000.0, 100.0, "ms"),
                PluginParameter::new(5, "Makeup Gain (dB)", 0.0, 24.0, 0.0, "dB"),
            ],
            compressor,
        }
    }
}

impl AudioPlugin for NativeCompressor {
    fn descriptor(&self) -> &PluginDescriptor {
        &self.descriptor
    }

    fn process(&mut self, left: &mut [f32], right: &mut [f32], _sidechain: Option<&[f32]>) {
        let _ = self.compressor.process(left, right);
    }

    fn reset(&mut self) {
        self.compressor.reset();
    }

    fn parameters(&self) -> &[PluginParameter] {
        &self.parameters
    }
}

/// Native 8-Band Parametric Channel EQ effect plugin
pub struct NativeChannelEq {
    descriptor: PluginDescriptor,
    parameters: Vec<PluginParameter>,
    eq: crate::dsp::equalizer::ChannelEq,
}

impl NativeChannelEq {
    pub fn new(sample_rate: f32) -> Self {
        let eq = crate::dsp::equalizer::ChannelEq::new(sample_rate).expect("Valid sample rate");
        Self {
            descriptor: PluginDescriptor::new(
                1003,
                "Channel EQ (8-Band)",
                PluginCategory::Equalizer,
            ),
            parameters: vec![
                PluginParameter::new(1, "Low Cut (Hz)", 20.0, 500.0, 30.0, "Hz"),
                PluginParameter::new(2, "High Cut (Hz)", 5000.0, 20000.0, 18000.0, "Hz"),
            ],
            eq,
        }
    }
}

impl AudioPlugin for NativeChannelEq {
    fn descriptor(&self) -> &PluginDescriptor {
        &self.descriptor
    }

    fn process(&mut self, left: &mut [f32], right: &mut [f32], _sidechain: Option<&[f32]>) {
        let _ = self.eq.process(left, right);
    }

    fn reset(&mut self) {
        // State clears automatically
    }

    fn parameters(&self) -> &[PluginParameter] {
        &self.parameters
    }
}

/// Native ChromaVerb algorithmic stereo reverb plugin
pub struct NativeChromaVerb {
    descriptor: PluginDescriptor,
    parameters: Vec<PluginParameter>,
    reverb: crate::dsp::reverb::ChromaVerbEngine,
}

impl NativeChromaVerb {
    pub fn new(sample_rate: f32) -> Self {
        let reverb =
            crate::dsp::reverb::ChromaVerbEngine::new(2.5, sample_rate).expect("Valid sample rate");
        Self {
            descriptor: PluginDescriptor::new(1005, "ChromaVerb", PluginCategory::Reverb),
            parameters: vec![
                PluginParameter::new(1, "Decay Time (s)", 0.2, 10.0, 2.5, "s"),
                PluginParameter::new(2, "Mix", 0.0, 1.0, 0.35, "%"),
            ],
            reverb,
        }
    }
}

impl AudioPlugin for NativeChromaVerb {
    fn descriptor(&self) -> &PluginDescriptor {
        &self.descriptor
    }

    fn process(&mut self, left: &mut [f32], right: &mut [f32], _sidechain: Option<&[f32]>) {
        let _ = self.reverb.process(left, right);
    }

    fn reset(&mut self) {
        // Cleared on next block
    }

    fn parameters(&self) -> &[PluginParameter] {
        &self.parameters
    }
}

/// Native Virtual Analog Subtractive Synthesizer
pub struct NativeSubtractiveSynth {
    descriptor: PluginDescriptor,
    parameters: Vec<PluginParameter>,
    sample_rate: f32,
    phase1: f32,
    phase2: f32,
    filter_state: [f32; 2],
}

impl NativeSubtractiveSynth {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            descriptor: PluginDescriptor::new(1007, "Subtractive Synth", PluginCategory::Synth),
            parameters: vec![
                PluginParameter::new(1, "Cutoff", 50.0, 18000.0, 2500.0, "Hz"),
                PluginParameter::new(2, "Resonance", 0.5, 8.0, 1.8, "Q"),
                PluginParameter::new(3, "Detune", -50.0, 50.0, 7.0, "cents"),
                PluginParameter::new(4, "Master Gain", 0.0, 1.0, 0.4, "%"),
            ],
            sample_rate,
            phase1: 0.0,
            phase2: 0.0,
            filter_state: [0.0; 2],
        }
    }
}

impl AudioPlugin for NativeSubtractiveSynth {
    fn descriptor(&self) -> &PluginDescriptor {
        &self.descriptor
    }

    fn process(&mut self, left: &mut [f32], right: &mut [f32], _sidechain: Option<&[f32]>) {
        let cutoff = self.parameters[0]
            .get_value()
            .clamp(20.0, self.sample_rate * 0.45);
        let q = self.parameters[1].get_value().clamp(0.5, 10.0);
        let detune_cents = self.parameters[2].get_value();
        let gain = self.parameters[3].get_value();

        let base_freq = 110.0f32; // A2
        let osc2_ratio = 2.0f32.powf(detune_cents / 1200.0);
        let dt1 = base_freq / self.sample_rate;
        let dt2 = (base_freq * osc2_ratio) / self.sample_rate;

        // 2-pole resonant lowpass filter coefficients
        let w0 = 2.0 * std::f32::consts::PI * cutoff / self.sample_rate;
        let alpha = (w0 * 0.5).sin() / q;
        let cos_w0 = w0.cos();
        let b1 = 1.0 - cos_w0;
        let b0 = b1 * 0.5;
        let b2 = b0;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos_w0;
        let a2 = 1.0 - alpha;

        let len = left.len().min(right.len());
        for i in 0..len {
            // Osc 1: Band-limited approximate saw (-1.0 to 1.0)
            let saw = 2.0 * self.phase1 - 1.0;
            // Osc 2: Square wave (-1.0 to 1.0)
            let square = if self.phase2 < 0.5 { 0.8 } else { -0.8 };

            let raw_voice = (saw * 0.6 + square * 0.4) * gain;

            // Direct Form I biquad filtering
            let filtered = (b0 / a0) * raw_voice
                + (b1 / a0) * self.filter_state[0]
                + (b2 / a0) * self.filter_state[1]
                - (a1 / a0) * self.filter_state[0]
                - (a2 / a0) * self.filter_state[1];

            self.filter_state[1] = self.filter_state[0];
            self.filter_state[0] = filtered.clamp(-2.0, 2.0);

            left[i] += self.filter_state[0];
            right[i] += self.filter_state[0];

            self.phase1 = (self.phase1 + dt1).fract();
            self.phase2 = (self.phase2 + dt2).fract();
        }
    }

    fn reset(&mut self) {
        self.phase1 = 0.0;
        self.phase2 = 0.0;
        self.filter_state = [0.0; 2];
    }

    fn parameters(&self) -> &[PluginParameter] {
        &self.parameters
    }
}

/// Native Percussion & Drum Synthesis Engine
pub struct NativeDrumSynth {
    descriptor: PluginDescriptor,
    parameters: Vec<PluginParameter>,
    sample_rate: f32,
    kick_phase: f32,
    kick_env: f32,
    noise_seed: u32,
}

impl NativeDrumSynth {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            descriptor: PluginDescriptor::new(1008, "Drum Synth", PluginCategory::Synth),
            parameters: vec![
                PluginParameter::new(1, "Kick Pitch", 35.0, 140.0, 55.0, "Hz"),
                PluginParameter::new(2, "Snare Snap", 0.0, 1.0, 0.6, "%"),
                PluginParameter::new(3, "Decay Time", 50.0, 800.0, 250.0, "ms"),
                PluginParameter::new(4, "Master Level", 0.0, 1.0, 0.5, "%"),
            ],
            sample_rate,
            kick_phase: 0.0,
            kick_env: 1.0,
            noise_seed: 0x12345678,
        }
    }

    fn next_noise(&mut self) -> f32 {
        self.noise_seed = self
            .noise_seed
            .wrapping_mul(1664525)
            .wrapping_add(1013904223);
        ((self.noise_seed >> 9) as f32 / 8388608.0) - 1.0
    }
}

impl AudioPlugin for NativeDrumSynth {
    fn descriptor(&self) -> &PluginDescriptor {
        &self.descriptor
    }

    fn process(&mut self, left: &mut [f32], right: &mut [f32], _sidechain: Option<&[f32]>) {
        let base_pitch = self.parameters[0].get_value();
        let snap = self.parameters[1].get_value();
        let decay_ms = self.parameters[2].get_value();
        let gain = self.parameters[3].get_value();

        let decay_coeff = (-1.0 / ((decay_ms / 1000.0) * self.sample_rate)).exp();
        let len = left.len().min(right.len());

        for i in 0..len {
            // Pitch drop envelope for punchy 808/909 kick
            let current_freq = base_pitch + (self.kick_env * 180.0);
            let dt = current_freq / self.sample_rate;
            self.kick_phase = (self.kick_phase + dt).fract();

            let kick_tone = (self.kick_phase * 2.0 * std::f32::consts::PI).sin() * self.kick_env;
            let noise = self.next_noise() * snap * self.kick_env.powi(2);

            let out_sample = (kick_tone + noise * 0.4) * gain;

            left[i] += out_sample;
            right[i] += out_sample;

            self.kick_env *= decay_coeff;
            if self.kick_env < 1e-4 {
                self.kick_env = 0.0;
            }
        }
    }

    fn reset(&mut self) {
        self.kick_phase = 0.0;
        self.kick_env = 1.0;
    }

    fn parameters(&self) -> &[PluginParameter] {
        &self.parameters
    }
}
