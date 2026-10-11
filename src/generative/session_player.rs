use crate::harmony::ChordTrack;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SessionPlayerType {
    #[default]
    Drummer,
    Bassist,
    Keyboardist,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GenerativePerformanceParams {
    pub complexity: f32, // 0.0 to 1.0 (X axis: simple -> complex)
    pub intensity: f32,  // 0.0 to 1.0 (Y axis: soft -> loud/aggressive)
    pub push_pull_ticks: i16, // Timing feel offset relative to beat grid (-120 to +120 ticks)
    pub fill_frequency: f32,  // 0.0 to 1.0
    pub swing: f32,           // 0.0 to 1.0
}

impl Default for GenerativePerformanceParams {
    fn default() -> Self {
        Self {
            complexity: 0.5,
            intensity: 0.7,
            push_pull_ticks: 0,
            fill_frequency: 0.25,
            swing: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GeneratedMidiNote {
    pub tick: u64,
    pub note_number: u8,
    pub velocity: u8,
    pub duration_ticks: u32,
}

pub struct SessionPlayer {
    pub player_type: SessionPlayerType,
    pub params: GenerativePerformanceParams,
}

impl SessionPlayer {
    pub fn new(player_type: SessionPlayerType, params: GenerativePerformanceParams) -> Self {
        Self { player_type, params }
    }

    pub fn generate_track(&self, chord_track: &ChordTrack, total_ticks: u64) -> Vec<GeneratedMidiNote> {
        let mut notes = Vec::new();
        let ppqn = 960u64;
        let step_ticks = ppqn / 2; // 8th note subdivisions
        let mut tick = 0u64;

        while tick < total_ticks {
            let chord = chord_track.get_chord_at(tick);
            let vel = ((self.params.intensity * 60.0) + 60.0).clamp(1.0, 127.0) as u8;
            let note_num = match self.player_type {
                SessionPlayerType::Bassist => 36 + (chord.bass % 12),
                SessionPlayerType::Drummer => {
                    // Kick on 1 and 3, Snare on 2 and 4
                    let beat_in_bar = (tick / ppqn) % 4;
                    if beat_in_bar == 0 || beat_in_bar == 2 { 36 } else { 38 }
                }
                SessionPlayerType::Keyboardist => 60 + (chord.root % 12),
            };

            let adjusted_tick = if self.params.push_pull_ticks < 0 {
                tick.saturating_sub((-self.params.push_pull_ticks) as u64)
            } else {
                tick + self.params.push_pull_ticks as u64
            };

            notes.push(GeneratedMidiNote {
                tick: adjusted_tick,
                note_number: note_num,
                velocity: vel,
                duration_ticks: (step_ticks * 3 / 4) as u32,
            });

            tick += step_ticks;
        }

        notes
    }
}
