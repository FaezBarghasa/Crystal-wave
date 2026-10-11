use crate::midi::midi_note::MidiNote;
use crate::timeline::MusicalScale;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct MidiTrackContainer {
    pub track_id: usize,
    pub name: String,
    pub notes: Vec<MidiNote>,
}

impl MidiTrackContainer {
    pub fn new(track_id: usize, name: impl Into<String>) -> Self {
        Self {
            track_id,
            name: name.into(),
            notes: Vec::new(),
        }
    }

    pub fn add_note(&mut self, note: MidiNote) {
        self.notes.push(note);
        self.notes.sort_by_key(|n| n.start_tick);
    }

    /// Scale quantize all notes to target scale and root pitch
    pub fn scale_quantize(&mut self, scale: MusicalScale, root_pitch: u8) {
        for note in &mut self.notes {
            note.pitch = scale.quantize_pitch(note.pitch, root_pitch);
        }
    }

    /// Demix by MIDI channel: splits notes into separate track containers per channel
    pub fn demix_by_channel(&self) -> HashMap<u8, Vec<MidiNote>> {
        let mut map: HashMap<u8, Vec<MidiNote>> = HashMap::new();
        for note in &self.notes {
            map.entry(note.channel).or_default().push(note.clone());
        }
        map
    }

    /// Demix by pitch: splits drum performances onto separate tracks per note number
    pub fn demix_by_pitch(&self) -> HashMap<u8, Vec<MidiNote>> {
        let mut map: HashMap<u8, Vec<MidiNote>> = HashMap::new();
        for note in &self.notes {
            map.entry(note.pitch).or_default().push(note.clone());
        }
        map
    }
}
