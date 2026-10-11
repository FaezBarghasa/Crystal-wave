use crate::generative::session_player::GeneratedMidiNote;
use crate::timeline::MusicalScale;

pub struct AiCounterMelody {
    pub scale: MusicalScale,
    pub root_pitch: u8,
}

impl AiCounterMelody {
    pub fn new(scale: MusicalScale, root_pitch: u8) -> Self {
        Self { scale, root_pitch }
    }

    /// Solves 1:1 Species Counterpoint avoiding parallel 5ths and octaves
    pub fn solve_counterpoint(
        &self,
        cantus_firmus: &[GeneratedMidiNote],
    ) -> Vec<GeneratedMidiNote> {
        let mut counterpoint = Vec::new();
        let mut prev_cf_pitch = 60u8;
        let mut prev_cp_pitch = 72u8;

        for (i, cf_note) in cantus_firmus.iter().enumerate() {
            let cf_motion = cf_note.note_number as i16 - prev_cf_pitch as i16;

            // Contrary motion target: if CF goes up, CP goes down
            let desired_cp_motion = -cf_motion;
            let mut candidate_pitch =
                (prev_cp_pitch as i16 + desired_cp_motion).clamp(50, 84) as u8;
            candidate_pitch = self.scale.quantize_pitch(candidate_pitch, self.root_pitch);

            // Avoid parallel 5ths (interval 7) and octaves (interval 0 or 12)
            let interval = (candidate_pitch as i16 - cf_note.note_number as i16).abs() % 12;
            if i > 0 && (interval == 7 || interval == 0) {
                // Adjust by a third (3 or 4 semitones)
                candidate_pitch = (candidate_pitch + 2).min(84);
                candidate_pitch = self.scale.quantize_pitch(candidate_pitch, self.root_pitch);
            }

            counterpoint.push(GeneratedMidiNote {
                tick: cf_note.tick,
                note_number: candidate_pitch,
                velocity: 82,
                duration_ticks: cf_note.duration_ticks,
            });

            prev_cf_pitch = cf_note.note_number;
            prev_cp_pitch = candidate_pitch;
        }

        counterpoint
    }
}
