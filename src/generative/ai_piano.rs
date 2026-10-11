use crate::generative::session_player::GeneratedMidiNote;
use crate::harmony::ChordTrack;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PianoCompingStyle {
    #[default]
    BlockChords,
    Arpeggio,
    Stride,
    Ballad,
}

pub struct AiPiano {
    pub style: PianoCompingStyle,
}

impl AiPiano {
    pub fn new(style: PianoCompingStyle) -> Self {
        Self { style }
    }

    pub fn generate_comping(
        &self,
        chord_track: &ChordTrack,
        bars: usize,
        ppqn: u64,
    ) -> Vec<GeneratedMidiNote> {
        let mut notes = Vec::new();
        let bar_ticks = ppqn * 4;

        for bar in 0..bars {
            let bar_start = bar as u64 * bar_ticks;
            let chord = chord_track.get_chord_at(bar_start);
            let chord_pitches = chord.notes();

            match self.style {
                PianoCompingStyle::BlockChords => {
                    // Chords on beats 1 and 3, or syncopated charleston rhythm
                    let times = [bar_start, bar_start + ppqn + (ppqn / 2)];
                    for &t in &times {
                        for &p in &chord_pitches {
                            notes.push(GeneratedMidiNote {
                                tick: t,
                                note_number: 60 + p,
                                velocity: 85,
                                duration_ticks: (ppqn * 3 / 4) as u32,
                            });
                        }
                    }
                }
                PianoCompingStyle::Arpeggio => {
                    // Ascending & descending 8th-note broken chords
                    for (i, &p) in chord_pitches.iter().cycle().take(8).enumerate() {
                        let t = bar_start + (i as u64 * ppqn / 2);
                        notes.push(GeneratedMidiNote {
                            tick: t,
                            note_number: 60 + p,
                            velocity: 80,
                            duration_ticks: (ppqn / 2) as u32,
                        });
                    }
                }
                PianoCompingStyle::Stride => {
                    // Beat 1: Bass root (octave 36); Beat 2: Mid-range chord (octave 60)
                    notes.push(GeneratedMidiNote {
                        tick: bar_start,
                        note_number: 36 + chord.bass,
                        velocity: 95,
                        duration_ticks: (ppqn * 3 / 4) as u32,
                    });
                    for &p in &chord_pitches {
                        notes.push(GeneratedMidiNote {
                            tick: bar_start + ppqn,
                            note_number: 60 + p,
                            velocity: 85,
                            duration_ticks: (ppqn * 3 / 4) as u32,
                        });
                    }
                }
                PianoCompingStyle::Ballad => {
                    // Whole note sustained chord with root in bass
                    notes.push(GeneratedMidiNote {
                        tick: bar_start,
                        note_number: 36 + chord.bass,
                        velocity: 80,
                        duration_ticks: (bar_ticks - 100) as u32,
                    });
                    for &p in &chord_pitches {
                        notes.push(GeneratedMidiNote {
                            tick: bar_start,
                            note_number: 60 + p,
                            velocity: 75,
                            duration_ticks: (bar_ticks - 100) as u32,
                        });
                    }
                }
            }
        }

        notes
    }
}
