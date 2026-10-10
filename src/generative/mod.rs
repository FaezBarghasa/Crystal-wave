use crate::harmony::{Chord, ChordTrack};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionPlayerType {
    Drummer,
    Bassist,
    Keyboardist,
}

#[derive(Debug, Clone)]
pub struct GenerativePerformanceParams {
    pub complexity: f32, // 0.0 (simple) to 1.0 (busy / complex)
    pub intensity: f32,  // 0.0 (soft) to 1.0 (loud / aggressive)
    pub swing: f32,      // 0.0 to 1.0 (50% to 75% swing)
    pub fill_frequency: f32, // 0.0 to 1.0
}

impl Default for GenerativePerformanceParams {
    fn default() -> Self {
        Self {
            complexity: 0.5,
            intensity: 0.7,
            swing: 0.0,
            fill_frequency: 0.3,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedMidiNote {
    pub tick: u64,
    pub note_number: u8,
    pub velocity: u8,
    pub duration_ticks: u32,
}

pub struct SessionPlayer {
    pub player_type: SessionPlayerType,
    pub params: GenerativePerformanceParams,
}

impl SessionPlayer {
    pub fn new(player_type: SessionPlayerType, params: GenerativePerformanceParams) -> Self {
        Self { player_type, params }
    }

    /// Generate performance notes for a given bar length (e.g. 4 bars = 4 * 960 * 4 ticks = 15360 ticks)
    pub fn generate_track(&self, chord_track: &ChordTrack, total_ticks: u64) -> Vec<GeneratedMidiNote> {
        match self.player_type {
            SessionPlayerType::Bassist => self.generate_bass_line(chord_track, total_ticks),
            SessionPlayerType::Drummer => self.generate_drum_groove(total_ticks),
            SessionPlayerType::Keyboardist => self.generate_keyboard_chords(chord_track, total_ticks),
        }
    }

    fn generate_bass_line(&self, chord_track: &ChordTrack, total_ticks: u64) -> Vec<GeneratedMidiNote> {
        let mut notes = Vec::new();
        let ppqn = 960u64; // Ticks per quarter note
        let step_ticks = ppqn / 2; // 8th note subdivisions

        let mut current_tick = 0;
        while current_tick < total_ticks {
            let chord = chord_track.chord_at_tick(current_tick).unwrap_or(Chord::new(0, crate::harmony::ChordQuality::Major));
            let root_pitch = 36 + chord.bass; // Low octave bass register (C2 = 36)

            // On downbeat (beat 1), always hit root
            let is_downbeat = (current_tick % (ppqn * 4)) == 0;
            let is_backbeat = (current_tick % ppqn) == 0;

            let note_pitch = if is_downbeat {
                root_pitch
            } else if is_backbeat && self.params.complexity > 0.4 {
                // Fifth of the chord
                root_pitch + 7
            } else if self.params.complexity > 0.7 {
                // Passing chromatic tone approaching next chord root
                (root_pitch + 2).min(60)
            } else {
                root_pitch
            };

            let vel = (40.0 + self.params.intensity * 70.0 + if is_downbeat { 15.0 } else { 0.0 }).clamp(1.0, 127.0) as u8;

            notes.push(GeneratedMidiNote {
                tick: current_tick,
                note_number: note_pitch,
                velocity: vel,
                duration_ticks: (step_ticks as f32 * 0.9) as u32,
            });

            current_tick += step_ticks;
        }

        notes
    }

    fn generate_drum_groove(&self, total_ticks: u64) -> Vec<GeneratedMidiNote> {
        let mut notes = Vec::new();
        let ppqn = 960u64;
        let sixteenth_tick = ppqn / 4; // 240 ticks

        let mut current_tick = 0;
        let mut step_index = 0;

        while current_tick < total_ticks {
            let beat_step = step_index % 16;
            let is_bar_end = (step_index % 16) >= 12;

            // Kick (MIDI 36): Beats 1 and 3 (step 0 and 8), plus syncopation based on complexity
            let has_kick = beat_step == 0 || beat_step == 8 || (self.params.complexity > 0.5 && beat_step == 10);
            if has_kick {
                let vel = (80.0 + self.params.intensity * 40.0) as u8;
                notes.push(GeneratedMidiNote {
                    tick: current_tick,
                    note_number: 36, // Kick
                    velocity: vel,
                    duration_ticks: 120,
                });
            }

            // Snare (MIDI 38): Beats 2 and 4 (step 4 and 12)
            let has_snare = beat_step == 4 || beat_step == 12;
            if has_snare {
                let vel = (85.0 + self.params.intensity * 40.0) as u8;
                notes.push(GeneratedMidiNote {
                    tick: current_tick,
                    note_number: 38, // Snare
                    velocity: vel,
                    duration_ticks: 120,
                });
            }

            // Hi-Hat (MIDI 42): 8th or 16th notes
            let has_hat = if self.params.complexity > 0.6 {
                true // 16th notes
            } else {
                beat_step % 2 == 0 // 8th notes
            };

            if has_hat {
                let vel = (50.0 + self.params.intensity * 35.0 - (beat_step % 2) as f32 * 15.0) as u8;
                notes.push(GeneratedMidiNote {
                    tick: current_tick,
                    note_number: 42, // Closed Hi-Hat
                    velocity: vel,
                    duration_ticks: 80,
                });
            }

            // Fill at bar end if fill_frequency triggered
            if is_bar_end && self.params.fill_frequency > 0.5 && (step_index % 32 >= 28) {
                notes.push(GeneratedMidiNote {
                    tick: current_tick,
                    note_number: 45, // Tom
                    velocity: (70.0 + self.params.intensity * 40.0) as u8,
                    duration_ticks: 100,
                });
            }

            current_tick += sixteenth_tick;
            step_index += 1;
        }

        notes
    }

    fn generate_keyboard_chords(&self, chord_track: &ChordTrack, total_ticks: u64) -> Vec<GeneratedMidiNote> {
        let mut notes = Vec::new();
        let ppqn = 960u64;

        let mut current_tick = 0;
        while current_tick < total_ticks {
            let chord = chord_track.chord_at_tick(current_tick).unwrap_or(Chord::new(0, crate::harmony::ChordQuality::Major));
            let chord_pitches = chord.notes();

            for pitch_class in chord_pitches {
                let midi_note = 60 + pitch_class; // Middle octave (C4 = 60)
                notes.push(GeneratedMidiNote {
                    tick: current_tick,
                    note_number: midi_note,
                    velocity: (65.0 + self.params.intensity * 35.0) as u8,
                    duration_ticks: (ppqn * 2) as u32,
                });
            }

            current_tick += ppqn * 2; // Half-note rhythmic harmonic comping
        }

        notes
    }
}
