use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChordQuality {
    Major,
    Minor,
    Diminished,
    Augmented,
    Dominant7,
    Major7,
    Minor7,
    Sus2,
    Sus4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Chord {
    pub root: u8, // 0 = C, 1 = C#, ..., 11 = B
    pub quality: ChordQuality,
    pub bass: u8,
}

impl Chord {
    pub fn new(root: u8, quality: ChordQuality) -> Self {
        Self {
            root: root % 12,
            quality,
            bass: root % 12,
        }
    }

    pub fn with_bass(mut self, bass: u8) -> Self {
        self.bass = bass % 12;
        self
    }

    /// Notes contained within this chord
    pub fn notes(&self) -> Vec<u8> {
        let r = self.root;
        let mut n = match self.quality {
            ChordQuality::Major => vec![r, (r + 4) % 12, (r + 7) % 12],
            ChordQuality::Minor => vec![r, (r + 3) % 12, (r + 7) % 12],
            ChordQuality::Diminished => vec![r, (r + 3) % 12, (r + 6) % 12],
            ChordQuality::Augmented => vec![r, (r + 4) % 12, (r + 8) % 12],
            ChordQuality::Dominant7 => vec![r, (r + 4) % 12, (r + 7) % 12, (r + 10) % 12],
            ChordQuality::Major7 => vec![r, (r + 4) % 12, (r + 7) % 12, (r + 11) % 12],
            ChordQuality::Minor7 => vec![r, (r + 3) % 12, (r + 7) % 12, (r + 10) % 12],
            ChordQuality::Sus2 => vec![r, (r + 2) % 12, (r + 7) % 12],
            ChordQuality::Sus4 => vec![r, (r + 5) % 12, (r + 7) % 12],
        };
        if !n.contains(&self.bass) {
            n.insert(0, self.bass);
        }
        n
    }
}

#[derive(Debug, Clone)]
pub struct TimedChord {
    pub start_tick: u64,
    pub duration_ticks: u64,
    pub chord: Chord,
}

#[derive(Debug, Clone, Default)]
pub struct ChordTrack {
    pub chords: Vec<TimedChord>,
    pub key_root: u8, // e.g. 0 = C
}

impl ChordTrack {
    pub fn new(key_root: u8) -> Self {
        Self {
            chords: Vec::new(),
            key_root: key_root % 12,
        }
    }

    pub fn add_chord(&mut self, start_tick: u64, duration_ticks: u64, chord: Chord) {
        self.chords.push(TimedChord {
            start_tick,
            duration_ticks,
            chord,
        });
        self.chords.sort_by_key(|c| c.start_tick);
    }

    pub fn chord_at_tick(&self, tick: u64) -> Option<Chord> {
        for tc in self.chords.iter().rev() {
            if tick >= tc.start_tick && tick < tc.start_tick + tc.duration_ticks {
                return Some(tc.chord);
            }
        }
        None
    }

    pub fn get_chord_at(&self, tick: u64) -> Chord {
        self.chord_at_tick(tick).unwrap_or(Chord::new(self.key_root, ChordQuality::Major))
    }
}

/// Variable-Length Markov Model (VLMM) for harmonic progressions (Logic MAHarmony engine)
pub struct MarkovProgressionEngine {
    /// Transition probability matrix: (CurrentChord, NextChord) -> Weight
    transitions: HashMap<(Chord, Chord), f32>,
}

impl Default for MarkovProgressionEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl MarkovProgressionEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            transitions: HashMap::new(),
        };
        engine.seed_diatonic_progressions();
        engine
    }

    /// Seed classical, jazz, and pop progression transitions (I - IV - V - I, ii - V - I, vi - IV - I - V)
    fn seed_diatonic_progressions(&mut self) {
        for root in 0..12 {
            let c_i = Chord::new(root, ChordQuality::Major);
            let c_ii = Chord::new((root + 2) % 12, ChordQuality::Minor);
            let c_iv = Chord::new((root + 5) % 12, ChordQuality::Major);
            let c_v = Chord::new((root + 7) % 12, ChordQuality::Dominant7);
            let c_vi = Chord::new((root + 9) % 12, ChordQuality::Minor);

            // Pop progression: I -> V -> vi -> IV
            self.add_transition(c_i, c_v, 0.35);
            self.add_transition(c_v, c_vi, 0.40);
            self.add_transition(c_vi, c_iv, 0.45);
            self.add_transition(c_iv, c_i, 0.40);

            // Jazz cadence: ii -> V -> I
            self.add_transition(c_ii, c_v, 0.60);
            self.add_transition(c_v, c_i, 0.70);
            self.add_transition(c_i, c_ii, 0.25);
            self.add_transition(c_i, c_iv, 0.35);
        }
    }

    pub fn add_transition(&mut self, from: Chord, to: Chord, weight: f32) {
        self.transitions.insert((from, to), weight);
    }

    /// Predict the most probable next chords given current chord
    pub fn predict_next_chords(&self, current: Chord) -> Vec<(Chord, f32)> {
        let mut matches: Vec<(Chord, f32)> = self
            .transitions
            .iter()
            .filter(|((from, _), _)| *from == current)
            .map(|((_, to), weight)| (*to, *weight))
            .collect();

        matches.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        matches
    }

    /// Calculate voice leading distance between two chords
    pub fn voice_leading_distance(c1: &Chord, c2: &Chord) -> f32 {
        let n1 = c1.notes();
        let n2 = c2.notes();
        let mut total_dist = 0;
        for note1 in &n1 {
            // Find closest note in n2
            let min_dist = n2.iter().map(|note2| {
                let diff = (*note1 as i32 - *note2 as i32).abs();
                diff.min(12 - diff)
            }).min().unwrap_or(0);
            total_dist += min_dist;
        }
        total_dist as f32 / n1.len() as f32
    }
}
