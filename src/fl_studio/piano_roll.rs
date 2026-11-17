/// Lower Event Editor Lane parameter target
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EventLaneTarget {
    #[default]
    Velocity,
    Pan,
    Pitch,
}

/// Musical Scale for Piano Roll Key Grid Highlighting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MusicalScale {
    #[default]
    Chromatic,
    Major,
    Minor,
    Dorian,
    Phrygian,
    Lydian,
    Mixolydian,
    HarmonicMinor,
    MelodicMinor,
    PentatonicMajor,
    PentatonicMinor,
}

impl MusicalScale {
    pub fn intervals(&self) -> &'static [u8] {
        match self {
            Self::Chromatic => &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
            Self::Major => &[0, 2, 4, 5, 7, 9, 11],
            Self::Minor => &[0, 2, 3, 5, 7, 8, 10],
            Self::Dorian => &[0, 2, 3, 5, 7, 9, 10],
            Self::Phrygian => &[0, 1, 3, 5, 7, 8, 10],
            Self::Lydian => &[0, 2, 4, 6, 7, 9, 11],
            Self::Mixolydian => &[0, 2, 4, 5, 7, 9, 10],
            Self::HarmonicMinor => &[0, 2, 3, 5, 7, 8, 11],
            Self::MelodicMinor => &[0, 2, 3, 5, 7, 9, 11],
            Self::PentatonicMajor => &[0, 2, 4, 7, 9],
            Self::PentatonicMinor => &[0, 3, 5, 7, 10],
        }
    }

    pub fn is_in_scale(&self, root_key: u8, key: u8) -> bool {
        if *self == Self::Chromatic {
            return true;
        }
        let semitones = (key as i16 - root_key as i16).rem_euclid(12) as u8;
        self.intervals().contains(&semitones)
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Chromatic => "Chromatic",
            Self::Major => "Major (Ionian)",
            Self::Minor => "Natural Minor (Aeolian)",
            Self::Dorian => "Dorian",
            Self::Phrygian => "Phrygian",
            Self::Lydian => "Lydian",
            Self::Mixolydian => "Mixolydian",
            Self::HarmonicMinor => "Harmonic Minor",
            Self::MelodicMinor => "Melodic Minor",
            Self::PentatonicMajor => "Major Pentatonic",
            Self::PentatonicMinor => "Minor Pentatonic",
        }
    }
}

/// Arpeggiator pattern directions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ArpDirection {
    #[default]
    Up,
    Down,
    UpDown,
    Random,
}

/// FL Studio-style Piano Roll Note with Slide, Portamento, Pan, and Fine Pitch
#[derive(Debug, Clone, PartialEq)]
pub struct FlPianoRollNote {
    pub id: usize,
    pub channel_id: usize,
    pub key: u8, // MIDI key 0-127
    pub start_tick: u64,
    pub length_ticks: u64,
    pub velocity: f32,  // 0.0 to 1.0
    pub pan: f32,       // -1.0 to +1.0
    pub is_slide: bool, // FL Studio slide triangle note: glides pitch of previous voice
    pub fine_pitch_cents: f32,
    pub color_group: u8, // 16 color channels in FL
}

impl FlPianoRollNote {
    pub fn new(id: usize, channel_id: usize, key: u8, start_tick: u64, length_ticks: u64) -> Self {
        Self {
            id,
            channel_id,
            key,
            start_tick,
            length_ticks,
            velocity: 0.78,
            pan: 0.0,
            is_slide: false,
            fine_pitch_cents: 0.0,
            color_group: 0,
        }
    }

    pub fn with_slide(mut self, is_slide: bool) -> Self {
        self.is_slide = is_slide;
        self
    }
}

/// Ghost Note representation from background inactive channels
#[derive(Debug, Clone, PartialEq)]
pub struct GhostNote {
    pub channel_id: usize,
    pub channel_name: String,
    pub key: u8,
    pub start_tick: u64,
    pub length_ticks: u64,
    pub opacity: f32, // Default 0.30 for calm-tech ghost visibility
}

/// FL Studio Advanced Alt-Menu Editing Tools
pub struct FlPianoRollTools;

impl FlPianoRollTools {
    /// Quick Strum: Staggers note start times in a chord to simulate realistic guitar/keyboard strumming
    pub fn quick_strum(
        notes: &mut [FlPianoRollNote],
        time_offset_ticks: u64,
        alternate_direction: bool,
    ) {
        notes.sort_by_key(|n| n.key);
        let len = notes.len();
        for (i, note) in notes.iter_mut().enumerate() {
            let offset = if alternate_direction && i % 2 == 1 {
                (len.saturating_sub(1 + i) as u64) * time_offset_ticks
            } else {
                (i as u64) * time_offset_ticks
            };
            note.start_tick += offset;
        }
    }

    /// Flip Horizontal: Reverses melody sequence across a time pivot
    pub fn flip_horizontal(notes: &mut [FlPianoRollNote], pivot_tick: u64) {
        for note in notes.iter_mut() {
            let distance = note.start_tick as i64 - pivot_tick as i64;
            let new_start = (pivot_tick as i64 - distance).max(0) as u64;
            note.start_tick = new_start;
        }
    }

    /// Flip Vertical: Inverts melody pitches across a pitch pivot key
    pub fn flip_vertical(notes: &mut [FlPianoRollNote], pivot_key: u8) {
        for note in notes.iter_mut() {
            let diff = note.key as i16 - pivot_key as i16;
            let inverted = (pivot_key as i16 - diff).clamp(0, 127) as u8;
            note.key = inverted;
        }
    }

    /// Chop / Arpeggiate: Divides notes into equal rhythmic subdivisions (e.g. 1/16, 1/32 for hi-hat rolls)
    pub fn quick_chop(note: &FlPianoRollNote, subdivisions: usize) -> Vec<FlPianoRollNote> {
        let subs = subdivisions.max(1);
        let sub_len = (note.length_ticks / subs as u64).max(1);
        let mut out = Vec::with_capacity(subs);

        for i in 0..subs {
            let mut sub_note = note.clone();
            sub_note.id = note.id * 1000 + i;
            sub_note.start_tick = note.start_tick + (i as u64) * sub_len;
            sub_note.length_ticks = sub_len;
            out.push(sub_note);
        }
        out
    }

    /// Advanced Strum with velocity taper and stroke alternation
    pub fn quick_strum_advanced(
        notes: &mut [FlPianoRollNote],
        time_offset_ticks: u64,
        alternate_direction: bool,
        velocity_taper: f32, // -0.5 to +0.5 taper
    ) {
        Self::quick_strum(notes, time_offset_ticks, alternate_direction);
        let len = notes.len();
        if len > 1 {
            for (i, note) in notes.iter_mut().enumerate() {
                let factor = 1.0 + velocity_taper * (i as f32 / (len - 1) as f32 - 0.5);
                note.velocity = (note.velocity * factor).clamp(0.05, 1.0);
            }
        }
    }

    /// Arpeggiate chord notes into sequential melodic patterns
    pub fn arpeggiate(
        chord_notes: &[FlPianoRollNote],
        direction: ArpDirection,
        step_ticks: u64,
        total_ticks: u64,
    ) -> Vec<FlPianoRollNote> {
        if chord_notes.is_empty() || step_ticks == 0 {
            return Vec::new();
        }

        let mut sorted = chord_notes.to_vec();
        sorted.sort_by_key(|n| n.key);

        let pitches: Vec<u8> = match direction {
            ArpDirection::Up => sorted.iter().map(|n| n.key).collect(),
            ArpDirection::Down => sorted.iter().rev().map(|n| n.key).collect(),
            ArpDirection::UpDown => {
                let mut p: Vec<u8> = sorted.iter().map(|n| n.key).collect();
                if sorted.len() > 2 {
                    let mut down: Vec<u8> = sorted[1..sorted.len() - 1]
                        .iter()
                        .rev()
                        .map(|n| n.key)
                        .collect();
                    p.append(&mut down);
                }
                p
            }
            ArpDirection::Random => {
                // Pseudo-random deterministic permutation based on pitch sum
                let mut p: Vec<u8> = sorted.iter().map(|n| n.key).collect();
                p.reverse();
                p
            }
        };

        let base_start = chord_notes.iter().map(|n| n.start_tick).min().unwrap_or(0);
        let channel_id = chord_notes[0].channel_id;
        let num_steps = (total_ticks / step_ticks).max(1);
        let note_len = step_ticks.saturating_sub(10).max(1);

        let mut out = Vec::with_capacity(num_steps as usize);
        for i in 0..num_steps {
            let key = pitches[(i as usize) % pitches.len()];
            let start = base_start + i * step_ticks;
            let mut arp_note = FlPianoRollNote::new(
                chord_notes[0].id * 1000 + i as usize,
                channel_id,
                key,
                start,
                note_len,
            );
            arp_note.velocity = chord_notes[0].velocity;
            out.push(arp_note);
        }
        out
    }
}
