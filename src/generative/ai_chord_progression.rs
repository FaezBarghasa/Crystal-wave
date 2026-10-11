use crate::harmony::{Chord, ChordQuality};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MusicalCadence {
    Authentic, // V -> I
    Plagal,    // IV -> I
    Deceptive, // V -> vi
    Half,      // I -> V
}

pub struct HarmonicTensionSolver;

impl HarmonicTensionSolver {
    /// Calculate harmonic tension score from 0.0 (resolution) to 1.0 (high tension/dominant)
    pub fn calculate_tension(chord: &Chord, key_root: u8) -> f32 {
        let relative_root = (chord.root + 12 - key_root) % 12;

        match relative_root {
            0 => 0.0, // Tonic: zero tension
            7 => match chord.quality {
                ChordQuality::Dominant7 => 0.95, // Dominant 7: high tension
                _ => 0.75,                       // V Major: strong tension
            },
            5 => 0.40, // Subdominant (IV): mild tension
            2 => 0.60, // Supertonic (ii): intermediate tension
            9 => 0.20, // Submediant (vi): tonic substitute
            11 => 0.98, // Leading tone (vii dim): maximum tension
            _ => 0.85, // Chromatic / borrowed chord: high tension
        }
    }

    /// Resolve cadence to tonic or deceptive chord
    pub fn create_cadence(cadence: MusicalCadence, key_root: u8) -> (Chord, Chord) {
        match cadence {
            MusicalCadence::Authentic => (
                Chord::new((key_root + 7) % 12, ChordQuality::Dominant7),
                Chord::new(key_root, ChordQuality::Major),
            ),
            MusicalCadence::Plagal => (
                Chord::new((key_root + 5) % 12, ChordQuality::Major),
                Chord::new(key_root, ChordQuality::Major),
            ),
            MusicalCadence::Deceptive => (
                Chord::new((key_root + 7) % 12, ChordQuality::Dominant7),
                Chord::new((key_root + 9) % 12, ChordQuality::Minor),
            ),
            MusicalCadence::Half => (
                Chord::new(key_root, ChordQuality::Major),
                Chord::new((key_root + 7) % 12, ChordQuality::Major),
            ),
        }
    }
}
