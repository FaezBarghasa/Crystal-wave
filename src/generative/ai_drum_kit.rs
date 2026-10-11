use crate::generative::session_player::GeneratedMidiNote;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DrumGenre {
    #[default]
    Rock,
    Pop,
    Jazz,
    Electronic,
    HipHop,
    Metal,
}

pub struct AiDrumKit {
    pub genre: DrumGenre,
    pub humanize: bool,
}

impl AiDrumKit {
    pub fn new(genre: DrumGenre) -> Self {
        Self { genre, humanize: true }
    }

    pub fn generate_pattern(&self, bars: usize, ppqn: u64) -> Vec<GeneratedMidiNote> {
        let mut notes = Vec::new();
        let bar_ticks = ppqn * 4;

        for bar in 0..bars {
            let bar_start = bar as u64 * bar_ticks;

            match self.genre {
                DrumGenre::Rock | DrumGenre::Pop => {
                    // Kick on 1 & 3
                    notes.push(GeneratedMidiNote { tick: bar_start, note_number: 36, velocity: 105, duration_ticks: 240 });
                    notes.push(GeneratedMidiNote { tick: bar_start + ppqn * 2, note_number: 36, velocity: 100, duration_ticks: 240 });
                    // Snare on 2 & 4
                    notes.push(GeneratedMidiNote { tick: bar_start + ppqn, note_number: 38, velocity: 110, duration_ticks: 240 });
                    notes.push(GeneratedMidiNote { tick: bar_start + ppqn * 3, note_number: 38, velocity: 115, duration_ticks: 240 });
                    // 8th-note Hi-Hats
                    for eighth in 0..8 {
                        let t = bar_start + (eighth as u64 * ppqn / 2);
                        let vel = if eighth % 2 == 0 { 90 } else { 75 };
                        notes.push(GeneratedMidiNote { tick: t, note_number: 42, velocity: vel, duration_ticks: 120 });
                    }
                }
                DrumGenre::Electronic | DrumGenre::HipHop => {
                    // 4 on the floor Kick for electronic or Boom-Bap kick
                    for beat in 0..4 {
                        notes.push(GeneratedMidiNote { tick: bar_start + beat as u64 * ppqn, note_number: 36, velocity: 115, duration_ticks: 240 });
                    }
                    // Claps/Snares on 2 & 4
                    notes.push(GeneratedMidiNote { tick: bar_start + ppqn, note_number: 39, velocity: 110, duration_ticks: 240 });
                    notes.push(GeneratedMidiNote { tick: bar_start + ppqn * 3, note_number: 39, velocity: 110, duration_ticks: 240 });
                    // 16th-note Hats with velocity rolls
                    for sixteenth in 0..16 {
                        let t = bar_start + (sixteenth as u64 * ppqn / 4);
                        notes.push(GeneratedMidiNote { tick: t, note_number: 42, velocity: 85, duration_ticks: 60 });
                    }
                }
                DrumGenre::Jazz => {
                    // Ride pattern: ding-ding-a-ding
                    for beat in 0..4 {
                        let t = bar_start + beat as u64 * ppqn;
                        notes.push(GeneratedMidiNote { tick: t, note_number: 51, velocity: 80, duration_ticks: 240 });
                        if beat == 1 || beat == 3 {
                            // Swung triplet skip note
                            let skip_t = t + (ppqn * 2 / 3);
                            notes.push(GeneratedMidiNote { tick: skip_t, note_number: 51, velocity: 70, duration_ticks: 120 });
                        }
                    }
                    // Feathered kick on all 4
                    for beat in 0..4 {
                        notes.push(GeneratedMidiNote { tick: bar_start + beat as u64 * ppqn, note_number: 36, velocity: 45, duration_ticks: 240 });
                    }
                    // Hi-Hat foot on 2 and 4
                    notes.push(GeneratedMidiNote { tick: bar_start + ppqn, note_number: 44, velocity: 80, duration_ticks: 120 });
                    notes.push(GeneratedMidiNote { tick: bar_start + ppqn * 3, note_number: 44, velocity: 80, duration_ticks: 120 });
                }
                DrumGenre::Metal => {
                    // Double bass 16th notes
                    for sixteenth in 0..16 {
                        let t = bar_start + (sixteenth as u64 * ppqn / 4);
                        notes.push(GeneratedMidiNote { tick: t, note_number: 36, velocity: 118, duration_ticks: 120 });
                    }
                    // Hard blast snare on 2 & 4
                    notes.push(GeneratedMidiNote { tick: bar_start + ppqn, note_number: 38, velocity: 125, duration_ticks: 200 });
                    notes.push(GeneratedMidiNote { tick: bar_start + ppqn * 3, note_number: 38, velocity: 125, duration_ticks: 200 });
                }
            }
        }

        notes
    }
}
