#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GrossBeatTimePreset {
    #[default]
    Off,
    HalfSpeed,
    Reverse1Bar,
    TapeStop,
    TripletStutter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GrossBeatVolumePreset {
    #[default]
    Off,
    SidechainPump,
    Gate16th,
    SawDecay,
}

/// Legendary Gross Beat Real-Time Time Manipulation & Gating Engine
pub struct GrossBeatEngine {
    buffer_l: Vec<f32>,
    buffer_r: Vec<f32>,
    write_pos: usize,
    capacity: usize,
    pub sample_rate: f32,
    pub tempo_bpm: f32,
    pub time_preset: GrossBeatTimePreset,
    pub volume_preset: GrossBeatVolumePreset,
    pub mix: f32, // 0.0 to 1.0
}

impl GrossBeatEngine {
    pub fn new(sample_rate: f32) -> Self {
        // 2 bars buffer at 120 BPM ~ 4.0 seconds max
        let capacity = (sample_rate * 4.0).round() as usize;
        Self {
            buffer_l: vec![0.0; capacity],
            buffer_r: vec![0.0; capacity],
            write_pos: 0,
            capacity,
            sample_rate,
            tempo_bpm: 120.0,
            time_preset: GrossBeatTimePreset::Off,
            volume_preset: GrossBeatVolumePreset::Off,
            mix: 1.0,
        }
    }

    pub fn process(&mut self, left: &mut [f32], right: &mut [f32]) {
        let len = left.len().min(right.len());
        if len == 0
            || (self.time_preset == GrossBeatTimePreset::Off
                && self.volume_preset == GrossBeatVolumePreset::Off)
        {
            return;
        }

        let cap = self.capacity;
        let samples_per_beat = ((self.sample_rate * 60.0) / self.tempo_bpm.max(10.0)).round() as usize;
        let quarter_window = samples_per_beat.max(1);
        let sixteenth_window = (samples_per_beat / 4).max(1);
        let eighth_window = (samples_per_beat / 2).max(1);
        let triplet_window = (samples_per_beat / 3).max(1);
        let stop_window = (samples_per_beat * 2).max(1);

        for i in 0..len {
            let in_l = left[i];
            let in_r = right[i];

            self.buffer_l[self.write_pos] = in_l;
            self.buffer_r[self.write_pos] = in_r;

            // 1. Time manipulation playback index
            let out_sample = match self.time_preset {
                GrossBeatTimePreset::Off => (in_l, in_r),
                GrossBeatTimePreset::HalfSpeed => {
                    // Half speed: read at half rate from previous half-bar
                    let read_offset = (self.write_pos / 2) % (cap / 2);
                    (self.buffer_l[read_offset], self.buffer_r[read_offset])
                }
                GrossBeatTimePreset::Reverse1Bar => {
                    let bar_len = (cap / 2).max(1);
                    let phase = self.write_pos % bar_len;
                    let read_idx = (self.write_pos + bar_len - phase) % cap;
                    (self.buffer_l[read_idx], self.buffer_r[read_idx])
                }
                GrossBeatTimePreset::TapeStop => {
                    // Exponential deceleration pitch drop
                    let decel = ((self.write_pos % stop_window) as f32 / stop_window as f32).powi(2);
                    let read_idx = ((self.write_pos as f32 * (1.0 - decel)) as usize) % cap;
                    (self.buffer_l[read_idx], self.buffer_r[read_idx])
                }
                GrossBeatTimePreset::TripletStutter => {
                    let read_idx = (self.write_pos / triplet_window * triplet_window) % cap;
                    (self.buffer_l[read_idx], self.buffer_r[read_idx])
                }
            };

            // 2. Volume Gating curve
            let vol_gain = match self.volume_preset {
                GrossBeatVolumePreset::Off => 1.0,
                GrossBeatVolumePreset::SidechainPump => {
                    let phase = (self.write_pos % quarter_window) as f32 / quarter_window as f32; // 1/4 note pump
                    (phase * std::f32::consts::FRAC_PI_2).sin()
                }
                GrossBeatVolumePreset::Gate16th => {
                    let phase = (self.write_pos % sixteenth_window) as f32 / sixteenth_window as f32; // 1/16 note
                    if phase < 0.5 { 1.0 } else { 0.0 }
                }
                GrossBeatVolumePreset::SawDecay => {
                    let phase = (self.write_pos % eighth_window) as f32 / eighth_window as f32;
                    1.0 - phase
                }
            };

            let wet_l = out_sample.0 * vol_gain;
            let wet_r = out_sample.1 * vol_gain;

            left[i] = in_l * (1.0 - self.mix) + wet_l * self.mix;
            right[i] = in_r * (1.0 - self.mix) + wet_r * self.mix;

            self.write_pos = (self.write_pos + 1) % cap;
        }
    }
}

impl crate::plugins::plugin_host::AudioPlugin for GrossBeatEngine {
    fn descriptor(&self) -> &crate::plugins::plugin_descriptor::PluginDescriptor {
        static DESC: std::sync::OnceLock<crate::plugins::plugin_descriptor::PluginDescriptor> =
            std::sync::OnceLock::new();
        DESC.get_or_init(|| {
            crate::plugins::plugin_descriptor::PluginDescriptor::new(
                1005,
                "Gross Beat",
                crate::plugins::plugin_descriptor::PluginCategory::Modulation,
            )
        })
    }

    fn process(&mut self, left: &mut [f32], right: &mut [f32], _sidechain: Option<&[f32]>) {
        GrossBeatEngine::process(self, left, right);
    }

    fn reset(&mut self) {
        self.buffer_l.fill(0.0);
        self.buffer_r.fill(0.0);
        self.write_pos = 0;
    }

    fn parameters(&self) -> &[crate::plugins::plugin_parameters::PluginParameter] {
        &[]
    }
}

