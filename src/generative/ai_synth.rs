use crate::generative::session_player::GeneratedMidiNote;
use crate::harmony::ChordTrack;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SynthPatternType {
    #[default]
    ArpeggiatorUp,
    ArpeggiatorDown,
    UpAndDown,
    TranceGate,
    LeadRiff,
}

pub struct AiSynth {
    pub pattern_type: SynthPatternType,
    pub octaves: u8,
}

impl AiSynth {
    pub fn new(pattern_type: SynthPatternType, octaves: u8) -> Self {
        Self {
            pattern_type,
            octaves: octaves.clamp(1, 3),
        }
    }

    pub fn generate_pattern(
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
            let base_pitches = chord.notes();

            // Expand across octaves
            let mut full_pitches = Vec::new();
            for oct in 0..self.octaves {
                for &p in &base_pitches {
                    full_pitches.push(60 + p + (oct * 12));
                }
            }

            if full_pitches.is_empty() {
                continue;
            }

            // 16th note sequencing (16 steps per bar)
            for step in 0..16 {
                let t = bar_start + (step as u64 * ppqn / 4);
                let pitch_idx = match self.pattern_type {
                    SynthPatternType::ArpeggiatorUp => step % full_pitches.len(),
                    SynthPatternType::ArpeggiatorDown => {
                        (full_pitches.len() - 1) - (step % full_pitches.len())
                    }
                    SynthPatternType::UpAndDown => {
                        let cycle_len = (full_pitches.len() * 2).saturating_sub(2).max(1);
                        let pos = step % cycle_len;
                        if pos < full_pitches.len() {
                            pos
                        } else {
                            cycle_len - pos
                        }
                    }
                    SynthPatternType::TranceGate => 0, // Repeated root with gated envelope
                    SynthPatternType::LeadRiff => (step * 3) % full_pitches.len(),
                };

                let note_num = full_pitches[pitch_idx % full_pitches.len()];
                notes.push(GeneratedMidiNote {
                    tick: t,
                    note_number: note_num,
                    velocity: 95,
                    duration_ticks: (ppqn / 6) as u32,
                });
            }
        }

        notes
    }
}
