use crate::generative::session_player::GeneratedMidiNote;
use crate::harmony::ChordTrack;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VocalRange {
    Soprano, // C4 to A5 (60..81)
    Alto,    // F3 to D5 (53..74)
    Tenor,   // C3 to A4 (48..69)
    #[default]
    Baritone,// G2 to E4 (43..64)
}

pub struct AiVocals {
    pub range: VocalRange,
}

impl AiVocals {
    pub fn new(range: VocalRange) -> Self {
        Self { range }
    }

    pub fn generate_lead_melody(&self, chord_track: &ChordTrack, bars: usize, ppqn: u64) -> Vec<GeneratedMidiNote> {
        let mut notes = Vec::new();
        let (min_pitch, max_pitch) = match self.range {
            VocalRange::Soprano => (60, 81),
            VocalRange::Alto => (53, 74),
            VocalRange::Tenor => (48, 69),
            VocalRange::Baritone => (43, 64),
        };

        for bar in 0..bars {
            // Bars 0, 1, 2 sing, bar 3 rests/breathes (typical 4-bar vocal phrasing)
            if bar % 4 == 3 {
                continue;
            }

            let bar_start = bar as u64 * (ppqn * 4);
            let chord = chord_track.get_chord_at(bar_start);
            let chord_pitches = chord.notes();

            for beat in 0..4 {
                let t = bar_start + beat as u64 * ppqn;
                let pitch_idx = (bar + beat) % chord_pitches.len();
                let pitch = (min_pitch + chord_pitches[pitch_idx]).clamp(min_pitch, max_pitch);

                notes.push(GeneratedMidiNote {
                    tick: t,
                    note_number: pitch,
                    velocity: 88,
                    duration_ticks: (ppqn * 3 / 4) as u32,
                });
            }
        }

        notes
    }
}
