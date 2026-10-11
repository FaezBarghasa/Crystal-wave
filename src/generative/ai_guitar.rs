use crate::generative::session_player::GeneratedMidiNote;
use crate::harmony::ChordTrack;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StrumDirection {
    #[default]
    Down,
    Up,
}

pub struct AiGuitar {
    pub strum_speed_ticks: u32, // Strum roll speed across strings (~20-40 ticks)
}

impl Default for AiGuitar {
    fn default() -> Self {
        Self {
            strum_speed_ticks: 24,
        }
    }
}

impl AiGuitar {
    pub fn new(strum_speed_ticks: u32) -> Self {
        Self { strum_speed_ticks }
    }

    pub fn generate_strum(
        &self,
        chord_track: &ChordTrack,
        tick: u64,
        direction: StrumDirection,
    ) -> Vec<GeneratedMidiNote> {
        let chord = chord_track.get_chord_at(tick);
        let mut pitches = chord.notes();
        if direction == StrumDirection::Up {
            pitches.reverse();
        }

        let mut notes = Vec::new();
        for (idx, &p) in pitches.iter().enumerate() {
            let string_tick = tick + (idx as u32 * self.strum_speed_ticks) as u64;
            notes.push(GeneratedMidiNote {
                tick: string_tick,
                note_number: 52 + p, // E2 guitar voicing range
                velocity: 90,
                duration_ticks: 960,
            });
        }

        notes
    }
}
