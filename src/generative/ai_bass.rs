use crate::generative::session_player::GeneratedMidiNote;
use crate::harmony::ChordTrack;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BassPlayStyle {
    #[default]
    Fingerstyle,
    Picked,
    Slap,
    Synth,
}

pub struct AiBass {
    pub style: BassPlayStyle,
    pub base_octave: u8, // MIDI 24, 36, etc.
}

impl AiBass {
    pub fn new(style: BassPlayStyle) -> Self {
        Self { style, base_octave: 36 }
    }

    pub fn generate_bassline(&self, chord_track: &ChordTrack, bars: usize, ppqn: u64) -> Vec<GeneratedMidiNote> {
        let mut notes = Vec::new();
        let bar_ticks = ppqn * 4;

        for bar in 0..bars {
            let bar_start = bar as u64 * bar_ticks;
            let chord = chord_track.get_chord_at(bar_start);
            let root = self.base_octave + (chord.root % 12);
            let fifth = root + 7;
            let octave = root + 12;

            match self.style {
                BassPlayStyle::Fingerstyle => {
                    // Beat 1: Root, Beat 2: Root, Beat 3: Fifth, Beat 4: Octave or passing
                    notes.push(GeneratedMidiNote { tick: bar_start, note_number: root, velocity: 95, duration_ticks: (ppqn * 3 / 4) as u32 });
                    notes.push(GeneratedMidiNote { tick: bar_start + ppqn, note_number: root, velocity: 85, duration_ticks: (ppqn / 2) as u32 });
                    notes.push(GeneratedMidiNote { tick: bar_start + ppqn * 2, note_number: fifth, velocity: 90, duration_ticks: (ppqn * 3 / 4) as u32 });
                    notes.push(GeneratedMidiNote { tick: bar_start + ppqn * 3, note_number: octave, velocity: 80, duration_ticks: (ppqn / 2) as u32 });
                }
                BassPlayStyle::Picked => {
                    // Driving 8th notes
                    for i in 0..8 {
                        let t = bar_start + (i as u64 * ppqn / 2);
                        let pitch = if i >= 6 { fifth } else { root };
                        notes.push(GeneratedMidiNote { tick: t, note_number: pitch, velocity: 100, duration_ticks: (ppqn / 3) as u32 });
                    }
                }
                BassPlayStyle::Slap => {
                    // Slap thumb on beat 1, pop index on upbeat
                    notes.push(GeneratedMidiNote { tick: bar_start, note_number: root, velocity: 115, duration_ticks: 200 });
                    notes.push(GeneratedMidiNote { tick: bar_start + ppqn / 2, note_number: octave, velocity: 120, duration_ticks: 150 });
                    notes.push(GeneratedMidiNote { tick: bar_start + ppqn * 2, note_number: fifth, velocity: 110, duration_ticks: 200 });
                }
                BassPlayStyle::Synth => {
                    // 16th-note rolling arpeggiated bassline
                    for i in 0..16 {
                        let t = bar_start + (i as u64 * ppqn / 4);
                        let pitch = if i % 2 == 0 { root } else { octave };
                        notes.push(GeneratedMidiNote { tick: t, note_number: pitch, velocity: 105, duration_ticks: 120 });
                    }
                }
            }
        }

        notes
    }
}
