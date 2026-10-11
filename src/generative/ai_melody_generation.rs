use crate::generative::session_player::GeneratedMidiNote;

pub struct MotifGenerator;

impl MotifGenerator {
    /// Invert motif around pivot pitch (inversion)
    pub fn invert(motif: &[GeneratedMidiNote], pivot_pitch: u8) -> Vec<GeneratedMidiNote> {
        motif
            .iter()
            .map(|n| {
                let diff = n.note_number as i16 - pivot_pitch as i16;
                let inverted_pitch = (pivot_pitch as i16 - diff).clamp(0, 127) as u8;
                GeneratedMidiNote {
                    tick: n.tick,
                    note_number: inverted_pitch,
                    velocity: n.velocity,
                    duration_ticks: n.duration_ticks,
                }
            })
            .collect()
    }

    /// Reverse motif notes in time (retrograde)
    pub fn retrograde(motif: &[GeneratedMidiNote]) -> Vec<GeneratedMidiNote> {
        if motif.is_empty() {
            return Vec::new();
        }
        let total_span = motif.last().unwrap().tick - motif[0].tick;
        let base_tick = motif[0].tick;

        motif
            .iter()
            .rev()
            .enumerate()
            .map(|(i, n)| {
                let offset = if i < motif.len() {
                    total_span - (n.tick - base_tick)
                } else {
                    0
                };
                GeneratedMidiNote {
                    tick: base_tick + offset,
                    note_number: n.note_number,
                    velocity: n.velocity,
                    duration_ticks: n.duration_ticks,
                }
            })
            .collect()
    }

    /// Double rhythmic duration (augmentation)
    pub fn augment(motif: &[GeneratedMidiNote]) -> Vec<GeneratedMidiNote> {
        if motif.is_empty() {
            return Vec::new();
        }
        let base_tick = motif[0].tick;
        motif
            .iter()
            .map(|n| {
                let offset = (n.tick - base_tick) * 2;
                GeneratedMidiNote {
                    tick: base_tick + offset,
                    note_number: n.note_number,
                    velocity: n.velocity,
                    duration_ticks: n.duration_ticks * 2,
                }
            })
            .collect()
    }

    /// Halve rhythmic duration (diminution)
    pub fn diminish(motif: &[GeneratedMidiNote]) -> Vec<GeneratedMidiNote> {
        if motif.is_empty() {
            return Vec::new();
        }
        let base_tick = motif[0].tick;
        motif
            .iter()
            .map(|n| {
                let offset = (n.tick - base_tick) / 2;
                GeneratedMidiNote {
                    tick: base_tick + offset,
                    note_number: n.note_number,
                    velocity: n.velocity,
                    duration_ticks: (n.duration_ticks / 2).max(1),
                }
            })
            .collect()
    }
}
