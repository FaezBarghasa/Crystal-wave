use crate::generative::session_player::GeneratedMidiNote;
use crate::harmony::Chord;

#[derive(Debug, Clone, PartialEq)]
pub struct SatbVoicing {
    pub soprano: GeneratedMidiNote,
    pub alto: GeneratedMidiNote,
    pub tenor: GeneratedMidiNote,
    pub bass: GeneratedMidiNote,
}

pub struct FourPartHarmonizer;

impl FourPartHarmonizer {
    /// Harmonize soprano note to full SATB choral voicing given chord harmony
    pub fn harmonize_satb(soprano_note: &GeneratedMidiNote, chord: &Chord) -> SatbVoicing {
        let root = chord.root;
        let third = (root + match chord.quality {
            crate::harmony::ChordQuality::Minor | crate::harmony::ChordQuality::Minor7 => 3,
            _ => 4,
        }) % 12;
        let fifth = (root + 7) % 12;

        let tick = soprano_note.tick;
        let dur = soprano_note.duration_ticks;

        // Bass voice: lowest root (octave 36)
        let bass_pitch = 36 + chord.bass;
        // Tenor voice: fifth or root in octave 48
        let tenor_pitch = 48 + fifth;
        // Alto voice: third in octave 60
        let alto_pitch = 60 + third;

        SatbVoicing {
            soprano: soprano_note.clone(),
            alto: GeneratedMidiNote { tick, note_number: alto_pitch, velocity: 80, duration_ticks: dur },
            tenor: GeneratedMidiNote { tick, note_number: tenor_pitch, velocity: 80, duration_ticks: dur },
            bass: GeneratedMidiNote { tick, note_number: bass_pitch, velocity: 90, duration_ticks: dur },
        }
    }
}
