use crate::generative::session_player::GeneratedMidiNote;
use crate::timeline::MusicalScale;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MelodicContour {
    #[default]
    Arch,       // Rise then fall
    Ascending,  // Climax at the end
    Descending, // Cascading down
    Wave,       // Oscillating
    Stationary, // Centered around pedal tone
}

pub struct AiMelody {
    pub contour: MelodicContour,
    pub scale: MusicalScale,
    pub root_pitch: u8,
}

impl AiMelody {
    pub fn new(contour: MelodicContour, scale: MusicalScale, root_pitch: u8) -> Self {
        Self { contour, scale, root_pitch }
    }

    pub fn generate_melody(&self, num_notes: usize, start_tick: u64, step_ticks: u32) -> Vec<GeneratedMidiNote> {
        let mut notes = Vec::new();
        if num_notes == 0 {
            return notes;
        }

        let intervals = self.scale.intervals();
        for i in 0..num_notes {
            let progress = i as f32 / (num_notes.saturating_sub(1).max(1) as f32);
            let contour_factor = match self.contour {
                MelodicContour::Arch => (progress * std::f32::consts::PI).sin(),
                MelodicContour::Ascending => progress,
                MelodicContour::Descending => 1.0 - progress,
                MelodicContour::Wave => (progress * 4.0 * std::f32::consts::PI).sin() * 0.5 + 0.5,
                MelodicContour::Stationary => 0.5,
            };

            let interval_idx = ((contour_factor * (intervals.len() - 1) as f32).round() as usize).min(intervals.len() - 1);
            let pitch = 60 + (self.root_pitch % 12) + intervals[interval_idx];

            notes.push(GeneratedMidiNote {
                tick: start_tick + (i as u32 * step_ticks) as u64,
                note_number: pitch,
                velocity: 90,
                duration_ticks: (step_ticks * 3 / 4),
            });
        }

        notes
    }
}
