use crate::harmony::{Chord, ChordQuality, ChordTrack};

pub struct AiChords;

impl AiChords {
    /// Generate standard pop/jazz chord progression: e.g. I - V - vi - IV
    pub fn generate_pop_four_chord(root: u8, bar_ticks: u64) -> ChordTrack {
        let mut track = ChordTrack::new(root);
        // I: Major
        track.add_chord(0, bar_ticks, Chord::new(root, ChordQuality::Major));
        // V: Major
        track.add_chord(
            bar_ticks,
            bar_ticks,
            Chord::new((root + 7) % 12, ChordQuality::Major),
        );
        // vi: Minor
        track.add_chord(
            bar_ticks * 2,
            bar_ticks,
            Chord::new((root + 9) % 12, ChordQuality::Minor),
        );
        // IV: Major
        track.add_chord(
            bar_ticks * 3,
            bar_ticks,
            Chord::new((root + 5) % 12, ChordQuality::Major),
        );
        track
    }

    /// Generate 2-5-1 Jazz turnaround: ii7 - V7 - Imaj7
    pub fn generate_ii_v_i(root: u8, bar_ticks: u64) -> ChordTrack {
        let mut track = ChordTrack::new(root);
        // ii7
        track.add_chord(
            0,
            bar_ticks,
            Chord::new((root + 2) % 12, ChordQuality::Minor7),
        );
        // V7
        track.add_chord(
            bar_ticks,
            bar_ticks,
            Chord::new((root + 7) % 12, ChordQuality::Dominant7),
        );
        // Imaj7
        track.add_chord(
            bar_ticks * 2,
            bar_ticks,
            Chord::new(root, ChordQuality::Major7),
        );
        track
    }
}
