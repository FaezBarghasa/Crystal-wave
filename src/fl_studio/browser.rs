use crate::fl_studio::channel_rack::ChannelRack;
use crate::plugins::plugin_descriptor::PluginCategory;
use crate::timeline::region::{Region, RegionKind};
use crate::timeline::track::Track;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BrowserTab {
    #[default]
    All,
    Drums,
    Bass,
    Instruments,
    Fx,
    Plugins,
    Presets,
}

impl BrowserTab {
    pub fn label(&self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Drums => "Drums",
            Self::Bass => "808/Bass",
            Self::Instruments => "Instruments",
            Self::Fx => "FX",
            Self::Plugins => "Plugins",
            Self::Presets => "Presets",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SampleEntry {
    pub id: usize,
    pub name: String,
    pub category: String, // "808", "Kicks", "Snares", "HiHats", "FX", "Instruments", "Vocals"
    pub file_path: String,
    pub duration_seconds: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PresetEntry {
    pub id: usize,
    pub name: String,
    pub category: String, // "Bass", "Lead", "Pluck", "Pad", "Mastering"
    pub plugin_name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PluginCatalogItem {
    pub id: u32,
    pub name: String,
    pub vendor: String,
    pub category: PluginCategory,
    pub is_generator: bool, // true = Synth/Instrument/Sampler, false = Insert FX
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct FlBrowser {
    pub samples: Vec<SampleEntry>,
    pub presets: Vec<PresetEntry>,
    pub plugins: Vec<PluginCatalogItem>,
    pub search_query: String,
    pub active_tab: BrowserTab,
    pub previewing_sample_id: Option<usize>,
}

impl Default for FlBrowser {
    fn default() -> Self {
        Self {
            samples: vec![
                SampleEntry {
                    id: 1,
                    name: "Spinz 808.wav".into(),
                    category: "808".into(),
                    file_path: "/samples/808/Spinz 808.wav".into(),
                    duration_seconds: 2.4,
                },
                SampleEntry {
                    id: 2,
                    name: "Rack Kick.wav".into(),
                    category: "Kicks".into(),
                    file_path: "/samples/kicks/Rack Kick.wav".into(),
                    duration_seconds: 0.8,
                },
                SampleEntry {
                    id: 3,
                    name: "Pierre Snare.wav".into(),
                    category: "Snares".into(),
                    file_path: "/samples/snares/Pierre Snare.wav".into(),
                    duration_seconds: 0.5,
                },
                SampleEntry {
                    id: 4,
                    name: "Lex Luger Hi-Hat.wav".into(),
                    category: "HiHats".into(),
                    file_path: "/samples/hats/Lex Luger Hi-Hat.wav".into(),
                    duration_seconds: 0.3,
                },
                SampleEntry {
                    id: 5,
                    name: "Sub Bass C1.wav".into(),
                    category: "808".into(),
                    file_path: "/samples/808/Sub Bass C1.wav".into(),
                    duration_seconds: 3.2,
                },
                SampleEntry {
                    id: 6,
                    name: "Acoustic Punch Kick.wav".into(),
                    category: "Kicks".into(),
                    file_path: "/samples/kicks/Acoustic Punch Kick.wav".into(),
                    duration_seconds: 0.9,
                },
                SampleEntry {
                    id: 7,
                    name: "Tight Trap Snare.wav".into(),
                    category: "Snares".into(),
                    file_path: "/samples/snares/Tight Trap Snare.wav".into(),
                    duration_seconds: 0.45,
                },
                SampleEntry {
                    id: 8,
                    name: "Laser Sweep FX.wav".into(),
                    category: "FX".into(),
                    file_path: "/samples/fx/Laser Sweep FX.wav".into(),
                    duration_seconds: 1.8,
                },
                SampleEntry {
                    id: 9,
                    name: "Rhodes Chords Cmaj7.wav".into(),
                    category: "Instruments".into(),
                    file_path: "/samples/instruments/Rhodes Chords Cmaj7.wav".into(),
                    duration_seconds: 4.0,
                },
                SampleEntry {
                    id: 10,
                    name: "Vocal Chant Hey.wav".into(),
                    category: "Vocals".into(),
                    file_path: "/samples/vocals/Vocal Chant Hey.wav".into(),
                    duration_seconds: 0.6,
                },
            ],
            presets: vec![
                PresetEntry {
                    id: 1,
                    name: "Bell Chime Pluck".into(),
                    category: "Pluck".into(),
                    plugin_name: "Sytrus".into(),
                },
                PresetEntry {
                    id: 2,
                    name: "Supersaw Anthem".into(),
                    category: "Lead".into(),
                    plugin_name: "3x OSC".into(),
                },
                PresetEntry {
                    id: 3,
                    name: "Dark Reese Bass".into(),
                    category: "Bass".into(),
                    plugin_name: "FLEX".into(),
                },
                PresetEntry {
                    id: 4,
                    name: "Lush Ambient Hall".into(),
                    category: "Reverb".into(),
                    plugin_name: "ChromaVerb".into(),
                },
                PresetEntry {
                    id: 5,
                    name: "Clean Master Ceiling".into(),
                    category: "Mastering".into(),
                    plugin_name: "Fruity Soft Clipper".into(),
                },
            ],
            plugins: vec![
                PluginCatalogItem {
                    id: 2001,
                    name: "3x OSC".into(),
                    vendor: "Image-Line / CrystalWave".into(),
                    category: PluginCategory::Synth,
                    is_generator: true,
                    description: "Subtractive 3-oscillator synth with stereo detune and noise".into(),
                },
                PluginCatalogItem {
                    id: 2002,
                    name: "Sytrus".into(),
                    vendor: "Image-Line / CrystalWave".into(),
                    category: PluginCategory::Synth,
                    is_generator: true,
                    description: "6-operator FM and RM synthesis matrix with ring mod".into(),
                },
                PluginCatalogItem {
                    id: 2003,
                    name: "FLEX".into(),
                    vendor: "Image-Line / CrystalWave".into(),
                    category: PluginCategory::Sampler,
                    is_generator: true,
                    description: "Preset-focused ROMpler synth with macro controls".into(),
                },
                PluginCatalogItem {
                    id: 2004,
                    name: "FPC Drum Sampler".into(),
                    vendor: "Image-Line / CrystalWave".into(),
                    category: PluginCategory::Sampler,
                    is_generator: true,
                    description: "16-pad MPC-style drum sampler with velocity layers".into(),
                },
                PluginCatalogItem {
                    id: 1004,
                    name: "Fruity Soft Clipper".into(),
                    vendor: "Image-Line / CrystalWave".into(),
                    category: PluginCategory::Dynamics,
                    is_generator: false,
                    description: "Hyperbolic tangent saturation with strict -0.1 dBFS ceiling".into(),
                },
                PluginCatalogItem {
                    id: 1006,
                    name: "Gross Beat".into(),
                    vendor: "Image-Line / CrystalWave".into(),
                    category: PluginCategory::Utility,
                    is_generator: false,
                    description: "Time and volume gating buffer with half-speed and tape stop".into(),
                },
                PluginCatalogItem {
                    id: 1005,
                    name: "ChromaVerb".into(),
                    vendor: "Apple / PureRust".into(),
                    category: PluginCategory::Reverb,
                    is_generator: false,
                    description: "Algorithmic circular reverb with high-damping room simulation".into(),
                },
                PluginCatalogItem {
                    id: 1001,
                    name: "Tape Delay".into(),
                    vendor: "Apple / PureRust".into(),
                    category: PluginCategory::Delay,
                    is_generator: false,
                    description: "Analog tape delay with wow/flutter tape emulation".into(),
                },
                PluginCatalogItem {
                    id: 1003,
                    name: "Channel EQ".into(),
                    vendor: "Apple / PureRust".into(),
                    category: PluginCategory::Equalizer,
                    is_generator: false,
                    description: "8-band parametric EQ with high/low cut and shelving".into(),
                },
                PluginCatalogItem {
                    id: 1002,
                    name: "Studio Compressor".into(),
                    vendor: "Apple / PureRust".into(),
                    category: PluginCategory::Dynamics,
                    is_generator: false,
                    description: "Classic VCA/Opto compressor with auto-release".into(),
                },
            ],
            search_query: String::new(),
            active_tab: BrowserTab::All,
            previewing_sample_id: None,
        }
    }
}

impl FlBrowser {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start_preview(&mut self, sample_id: usize) {
        self.previewing_sample_id = Some(sample_id);
    }

    pub fn stop_preview(&mut self) {
        self.previewing_sample_id = None;
    }

    /// Instant Smart Search: matches query substring against sample name and category
    pub fn search(&self, query: &str) -> Vec<&SampleEntry> {
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            return self.samples.iter().collect();
        }

        self.samples
            .iter()
            .filter(|s| {
                s.name.to_lowercase().contains(&q) || s.category.to_lowercase().contains(&q)
            })
            .collect()
    }

    /// Filtered samples based on current active tab and search query
    pub fn filtered_samples(&self) -> Vec<&SampleEntry> {
        let q = self.search_query.trim().to_lowercase();
        self.samples
            .iter()
            .filter(|s| {
                let matches_tab = match self.active_tab {
                    BrowserTab::All => true,
                    BrowserTab::Drums => {
                        s.category == "Kicks" || s.category == "Snares" || s.category == "HiHats"
                    }
                    BrowserTab::Bass => s.category == "808",
                    BrowserTab::Instruments => s.category == "Instruments" || s.category == "Vocals",
                    BrowserTab::Fx => s.category == "FX",
                    BrowserTab::Plugins | BrowserTab::Presets => false,
                };
                let matches_query = if q.is_empty() {
                    true
                } else {
                    s.name.to_lowercase().contains(&q) || s.category.to_lowercase().contains(&q)
                };
                matches_tab && matches_query
            })
            .collect()
    }

    /// Filtered presets based on search query
    pub fn filtered_presets(&self) -> Vec<&PresetEntry> {
        let q = self.search_query.trim().to_lowercase();
        self.presets
            .iter()
            .filter(|p| {
                if q.is_empty() {
                    true
                } else {
                    p.name.to_lowercase().contains(&q)
                        || p.category.to_lowercase().contains(&q)
                        || p.plugin_name.to_lowercase().contains(&q)
                }
            })
            .collect()
    }

    /// Filtered plugins based on search query and category
    pub fn filtered_plugins(
        &self,
        search_query: &str,
        category: Option<PluginCategory>,
    ) -> Vec<&PluginCatalogItem> {
        let q = search_query.trim().to_lowercase();
        self.plugins
            .iter()
            .filter(|p| {
                let matches_cat = category.is_none_or(|c| p.category == c);
                let matches_query = if q.is_empty() {
                    true
                } else {
                    p.name.to_lowercase().contains(&q)
                        || p.vendor.to_lowercase().contains(&q)
                        || p.description.to_lowercase().contains(&q)
                };
                matches_cat && matches_query
            })
            .collect()
    }

    /// Direct Drag-and-Drop: Instantly loads sample into Channel Rack
    pub fn drop_to_channel_rack(&self, rack: &mut ChannelRack, sample_id: usize) -> Option<usize> {
        let sample = self.samples.iter().find(|s| s.id == sample_id)?;
        let chan_id = rack.add_channel(sample.name.clone());
        if let Some(chan) = rack.channels.get_mut(chan_id) {
            chan.sample_path = Some(sample.file_path.clone());
        }
        Some(chan_id)
    }

    /// Direct Drag-and-Drop: Instantly stamps sample as an Audio Region onto a Playlist Track
    pub fn drop_to_playlist(
        &self,
        tracks: &mut [Track],
        sample_id: usize,
        track_id: usize,
        start_tick: u64,
        length_ticks: u64,
    ) -> Option<usize> {
        let sample = self.samples.iter().find(|s| s.id == sample_id)?;
        let max_id = tracks
            .iter()
            .flat_map(|t| t.regions.iter())
            .map(|r| r.id)
            .max()
            .unwrap_or(0);
        let new_id = max_id + 1;

        if let Some(track) = tracks.get_mut(track_id) {
            let sample_len = (sample.duration_seconds * 44100.0) as usize;
            track.regions.push(Region {
                id: new_id,
                track_id,
                name: sample.name.clone(),
                start_tick,
                length_ticks,
                anchor_offset_ticks: 0,
                muted: false,
                looped: false,
                loop_length_ticks: None,
                gain_db: 0.0,
                transpose_semitones: 0.0,
                reversed: false,
                fade: Default::default(),
                kind: RegionKind::Audio {
                    file_path: Some(sample.file_path.clone()),
                    sample_offset: 0,
                    sample_length: sample_len,
                },
            });
            Some(new_id)
        } else {
            None
        }
    }
}
