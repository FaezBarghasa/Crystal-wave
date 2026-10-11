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

    #[inline(always)]
    fn read_interpolated(&self, read_pos: f32) -> (f32, f32) {
        let cap = self.capacity;
        let pos = read_pos.rem_euclid(cap as f32);
        let i0 = pos.floor() as usize % cap;
        let i1 = (i0 + 1) % cap;
        let frac = pos - pos.floor();
        let l = self.buffer_l[i0] * (1.0 - frac) + self.buffer_l[i1] * frac;
        let r = self.buffer_r[i0] * (1.0 - frac) + self.buffer_r[i1] * frac;
        (l, r)
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
        let samples_per_beat =
            ((self.sample_rate * 60.0) / self.tempo_bpm.max(10.0)).round() as usize;
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

            // 1. Time manipulation with interpolated playback
            let out_sample = match self.time_preset {
                GrossBeatTimePreset::Off => (in_l, in_r),
                GrossBeatTimePreset::HalfSpeed => {
                    let read_offset = ((self.write_pos / 2) % (cap / 2)) as f32;
                    self.read_interpolated(read_offset)
                }
                GrossBeatTimePreset::Reverse1Bar => {
                    let bar_len = (cap / 2).max(1);
                    let phase = self.write_pos % bar_len;
                    let read_idx = ((self.write_pos + bar_len - phase) % cap) as f32;
                    self.read_interpolated(read_idx)
                }
                GrossBeatTimePreset::TapeStop => {
                    let decel =
                        ((self.write_pos % stop_window) as f32 / stop_window as f32).powi(2);
                    let read_idx = (self.write_pos as f32 * (1.0 - decel)).rem_euclid(cap as f32);
                    self.read_interpolated(read_idx)
                }
                GrossBeatTimePreset::TripletStutter => {
                    let read_idx =
                        ((self.write_pos / triplet_window * triplet_window) % cap) as f32;
                    self.read_interpolated(read_idx)
                }
            };

            // 2. Volume Gating curve with de-clicking edge smoothing
            let vol_gain = match self.volume_preset {
                GrossBeatVolumePreset::Off => 1.0,
                GrossBeatVolumePreset::SidechainPump => {
                    let phase = (self.write_pos % quarter_window) as f32 / quarter_window as f32;
                    (phase * std::f32::consts::FRAC_PI_2).sin()
                }
                GrossBeatVolumePreset::Gate16th => {
                    let phase =
                        (self.write_pos % sixteenth_window) as f32 / sixteenth_window as f32;
                    if phase < 0.05 {
                        phase / 0.05
                    } else if phase < 0.45 {
                        1.0
                    } else if phase < 0.50 {
                        1.0 - (phase - 0.45) / 0.05
                    } else {
                        0.0
                    }
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
