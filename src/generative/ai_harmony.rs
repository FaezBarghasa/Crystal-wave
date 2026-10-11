use crate::generative::session_player::GeneratedMidiNote;
use crate::timeline::MusicalScale;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HarmonyInterval {
    #[default]
    ThirdAbove,
    ThirdBelow,
    SixthBelow,
    OctaveAbove,
    ContraryMotion,
}

pub struct AiHarmony {
    pub interval: HarmonyInterval,
    pub scale: MusicalScale,
    pub root_pitch: u8,
}

impl AiHarmony {
    pub fn new(interval: HarmonyInterval, scale: MusicalScale, root_pitch: u8) -> Self {
        Self { interval, scale, root_pitch }
    }

    pub fn harmonize_melody(&self, lead_notes: &[GeneratedMidiNote]) -> Vec<GeneratedMidiNote> {
        let mut harm_notes = Vec::new();
        for (idx, note) in lead_notes.iter().enumerate() {
            let offset_semitones: i8 = match self.interval {
                HarmonyInterval::ThirdAbove => 4,
                HarmonyInterval::ThirdBelow => -3,
                HarmonyInterval::SixthBelow => -8,
                HarmonyInterval::OctaveAbove => 12,
                HarmonyInterval::ContraryMotion => {
                    // Move inversely to note contour
                    if idx % 2 == 0 { -4 } else { 4 }
                }
            };

            let raw_pitch = (note.note_number as i16 + offset_semitones as i16).clamp(0, 127) as u8;
            let scale_pitch = self.scale.quantize_pitch(raw_pitch, self.root_pitch);

            harm_notes.push(GeneratedMidiNote {
                tick: note.tick,
                note_number: scale_pitch,
                velocity: (note.velocity as f32 * 0.9).round() as u8,
                duration_ticks: note.duration_ticks,
            });
        }
        harm_notes
    }
}
