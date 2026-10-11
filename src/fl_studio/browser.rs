use crate::fl_studio::channel_rack::ChannelRack;

#[derive(Debug, Clone)]
pub struct SampleEntry {
    pub id: usize,
    pub name: String,
    pub category: String, // "808", "Kicks", "Snares", "HiHats", "FX", "Loops"
    pub file_path: String,
    pub duration_seconds: f32,
}

pub struct FlBrowser {
    pub samples: Vec<SampleEntry>,
    pub search_query: String,
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
            ],
            search_query: String::new(),
        }
    }
}

impl FlBrowser {
    pub fn new() -> Self {
        Self::default()
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

    /// Direct Drag-and-Drop: Instantly loads sample into Channel Rack
    pub fn drop_to_channel_rack(&self, rack: &mut ChannelRack, sample_id: usize) -> Option<usize> {
        let sample = self.samples.iter().find(|s| s.id == sample_id)?;
        let chan_id = rack.add_channel(sample.name.clone());
        if let Some(chan) = rack.channels.get_mut(chan_id) {
            chan.sample_path = Some(sample.file_path.clone());
        }
        Some(chan_id)
    }
}
