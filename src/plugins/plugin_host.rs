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
        let delay_samples = ((delay_ms / 1000.0) * 44100.0).round() as usize;
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
        for s in &mut self.delay_buffer_l { *s = 0.0; }
        for s in &mut self.delay_buffer_r { *s = 0.0; }
        self.write_pos = 0;
    }

    fn parameters(&self) -> &[PluginParameter] {
        &self.parameters
    }
}
