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
}
