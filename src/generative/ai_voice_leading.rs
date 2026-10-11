use crate::harmony::Chord;

pub struct VoiceLeadingSolver;

impl VoiceLeadingSolver {
    /// Calculate harmonic proximity distance between two voicings
    /// Evaluates pitch distances (semitone movement) across voices to minimize jump distance
    pub fn voice_distance(voicing_a: &[u8], voicing_b: &[u8]) -> u32 {
        let mut total_dist = 0u32;
        let len = voicing_a.len().min(voicing_b.len());
        for i in 0..len {
            total_dist += (voicing_a[i] as i16 - voicing_b[i] as i16).unsigned_abs() as u32;
        }
        total_dist
    }

    /// Evaluates optimal inversion for next chord to minimize voice movement
    /// (`UseForChordBassDistToHarmCenter` algorithm)
    pub fn optimal_inversion(
        current_voicing: &[u8],
        next_chord: &Chord,
        octave_base: u8,
    ) -> Vec<u8> {
        let base_notes = next_chord.notes(); // e.g. [root, 3rd, 5th]
        let mut best_voicing = Vec::new();
        let mut min_distance = u32::MAX;

        // Test root position, 1st inversion, 2nd inversion
        for inversion in 0..base_notes.len() {
            let mut candidate = Vec::new();
            for i in 0..base_notes.len() {
                let note_idx = (i + inversion) % base_notes.len();
                let oct_shift = if (i + inversion) >= base_notes.len() {
                    12
                } else {
                    0
                };
                candidate.push(octave_base + base_notes[note_idx] + oct_shift);
            }

            let dist = Self::voice_distance(current_voicing, &candidate);
            if dist < min_distance {
                min_distance = dist;
                best_voicing = candidate;
            }
        }

        best_voicing
    }
}
