use crate::generative::session_player::GeneratedMidiNote;
use crate::harmony::ChordTrack;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum KeyboardInstrumentType {
    #[default]
    RhodesElectricPiano,
    Wurlitzer,
    B3TonewheelOrgan,
    Clavinet,
}

pub struct AiKeyboards {
    pub instrument: KeyboardInstrumentType,
}

impl AiKeyboards {
    pub fn new(instrument: KeyboardInstrumentType) -> Self {
        Self { instrument }
    }

    pub fn generate_voicings(&self, chord_track: &ChordTrack, bars: usize, ppqn: u64) -> Vec<GeneratedMidiNote> {
        let mut notes = Vec::new();
        let bar_ticks = ppqn * 4;

        for bar in 0..bars {
            let bar_start = bar as u64 * bar_ticks;
            let chord = chord_track.get_chord_at(bar_start);
            let pitches = chord.notes();

            match self.instrument {
                KeyboardInstrumentType::RhodesElectricPiano | KeyboardInstrumentType::Wurlitzer => {
                    // Smooth close-voiced 4-note comping
                    let t = bar_start + (ppqn / 2);
                    for &p in &pitches {
                        notes.push(GeneratedMidiNote {
                            tick: t,
                            note_number: 60 + p,
                            velocity: 78,
                            duration_ticks: (ppqn * 2) as u32,
                        });
                    }
                }
                KeyboardInstrumentType::B3TonewheelOrgan => {
                    // Continuous sustained organ pads with swell
                    for &p in &pitches {
                        notes.push(GeneratedMidiNote {
                            tick: bar_start,
                            note_number: 60 + p,
                            velocity: 88,
                            duration_ticks: (bar_ticks - 50) as u32,
                        });
                    }
                }
                KeyboardInstrumentType::Clavinet => {
                    // Funk 16th-note staccato rhythm
                    for step in [0, 3, 6, 10, 12] {
                        let t = bar_start + (step as u64 * ppqn / 4);
                        for &p in &pitches {
                            notes.push(GeneratedMidiNote {
                                tick: t,
                                note_number: 60 + p,
                                velocity: 95,
                                duration_ticks: (ppqn / 8) as u32,
                            });
                        }
                    }
                }
            }
        }

        notes
    }
}
