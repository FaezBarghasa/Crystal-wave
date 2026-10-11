use crate::midi::midi_note::MidiNote;

#[derive(Debug, Clone, PartialEq)]
pub struct MidiDrawEvent {
    pub tick: u64,
    pub cc_number: u8,
    pub value: u16, // High resolution (14-bit or 7-bit)
}

#[derive(Debug, Clone, PartialEq)]
pub struct TabularMidiEvent {
    pub tick: u64,
    pub status_type: String,
    pub channel: u8,
    pub byte1: u8,
    pub byte2: u8,
    pub length_ticks: Option<u32>,
}

pub struct MidiEditor {
    pub draw_events: Vec<MidiDrawEvent>,
}

impl Default for MidiEditor {
    fn default() -> Self {
        Self {
            draw_events: Vec::new(),
        }
    }
}

impl MidiEditor {
    pub fn new() -> Self {
        Self::default()
    }

    /// Brush tool: paints notes repeating at regular grid division
    pub fn brush_notes(
        start_tick: u64,
        end_tick: u64,
        step_ticks: u32,
        pitch: u8,
        velocity_7bit: u8,
    ) -> Vec<MidiNote> {
        let mut notes = Vec::new();
        let mut current_tick = start_tick;
        let mut id = 0;
        let dur = (step_ticks * 3) / 4; // 75% gate

        while current_tick < end_tick {
            notes.push(MidiNote::new(id, current_tick, dur, pitch, velocity_7bit));
            id += 1;
            current_tick += step_ticks as u64;
        }

        notes
    }

    /// Humanize function: applies subtle pseudo-random jitter to tick, velocity, and duration
    pub fn humanize_notes(notes: &mut [MidiNote], tick_jitter: i32, vel_jitter: i16) {
        let mut seed = 123456789u64;
        for note in notes.iter_mut() {
            // LCG pseudo-random number generator
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let r1 = ((seed >> 32) as i32) % (tick_jitter.abs().max(1) * 2 + 1) - tick_jitter.abs();
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let r2 = ((seed >> 32) as i32) % (vel_jitter.abs().max(1) as i32 * 2 + 1) - vel_jitter.abs() as i32;

            if r1 < 0 {
                note.start_tick = note.start_tick.saturating_sub((-r1) as u64);
            } else {
                note.start_tick += r1 as u64;
            }

            let new_vel = (note.velocity as i32 + (r2 << 9)).clamp(1, 65535) as u16;
            note.velocity = new_vel;
        }
    }

    /// Flam function: adds a quick grace note preceding the main note
    pub fn apply_flam(note: &MidiNote, flam_offset_ticks: u32, flam_id: usize) -> (MidiNote, MidiNote) {
        let grace_tick = note.start_tick.saturating_sub(flam_offset_ticks as u64);
        let grace_vel = (note.velocity / 2).max(1000);
        let grace_note = MidiNote {
            id: flam_id,
            start_tick: grace_tick,
            duration_ticks: flam_offset_ticks.min(note.duration_ticks / 2).max(1),
            pitch: note.pitch,
            velocity: grace_vel,
            release_velocity: note.release_velocity,
            articulation_id: note.articulation_id,
            mpe: note.mpe.clone(),
            channel: note.channel,
            muted: false,
        };

        (grace_note, note.clone())
    }

    /// Generate Tabular Event List view
    pub fn to_event_list(notes: &[MidiNote]) -> Vec<TabularMidiEvent> {
        let mut list = Vec::new();
        for n in notes {
            list.push(TabularMidiEvent {
                tick: n.start_tick,
                status_type: "Note On".into(),
                channel: n.channel,
                byte1: n.pitch,
                byte2: (n.velocity >> 9) as u8,
                length_ticks: Some(n.duration_ticks),
            });
        }
        list.sort_by_key(|e| e.tick);
        list
    }
}
