use midir::{Ignore, MidiInput, MidiInputConnection, MidiInputPort};
use wmidi::MidiMessage;

#[derive(Debug, Clone, PartialEq)]
pub struct MidiDeviceInfo {
    pub id: usize,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParsedMidiEvent {
    NoteOn {
        channel: u8,
        note: u8,
        velocity: u8,
        timestamp_samples: u64,
    },
    NoteOff {
        channel: u8,
        note: u8,
        velocity: u8,
        timestamp_samples: u64,
    },
    ControlChange {
        channel: u8,
        controller: u8,
        value: u8,
        timestamp_samples: u64,
    },
    PitchBend {
        channel: u8,
        bend_raw: u16,
        bend_norm: f32, // -1.0 to 1.0
        timestamp_samples: u64,
    },
    PolyphonicAftertouch {
        channel: u8,
        note: u8,
        pressure: u8,
        timestamp_samples: u64,
    },
    ChannelAftertouch {
        channel: u8,
        pressure: u8,
        timestamp_samples: u64,
    },
    SystemRealtime(u8),
    Ignored,
}

/// Zero-allocation MIDI parser for real-time streams
pub struct RealtimeMidiParser;

impl RealtimeMidiParser {
    pub fn parse_slice(bytes: &[u8], timestamp_samples: u64) -> Option<ParsedMidiEvent> {
        if bytes.is_empty() {
            return None;
        }

        match MidiMessage::try_from(bytes) {
            Ok(msg) => match msg {
                MidiMessage::NoteOn(ch, note, vel) => {
                    let vel_u8: u8 = vel.into();
                    let note_u8: u8 = note.into();
                    let ch_u8 = ch.index();
                    // In MIDI spec, NoteOn with velocity 0 is treated as NoteOff
                    if vel_u8 == 0 {
                        Some(ParsedMidiEvent::NoteOff {
                            channel: ch_u8,
                            note: note_u8,
                            velocity: 0,
                            timestamp_samples,
                        })
                    } else {
                        Some(ParsedMidiEvent::NoteOn {
                            channel: ch_u8,
                            note: note_u8,
                            velocity: vel_u8,
                            timestamp_samples,
                        })
                    }
                }
                MidiMessage::NoteOff(ch, note, vel) => Some(ParsedMidiEvent::NoteOff {
                    channel: ch.index(),
                    note: note.into(),
                    velocity: vel.into(),
                    timestamp_samples,
                }),
                MidiMessage::ControlChange(ch, cc, val) => {
                    let cc_u8: u8 = cc.into();
                    let val_u8: u8 = val.into();
                    Some(ParsedMidiEvent::ControlChange {
                        channel: ch.index(),
                        controller: cc_u8,
                        value: val_u8,
                        timestamp_samples,
                    })
                }
                MidiMessage::PitchBendChange(ch, bend) => {
                    let raw: u16 = bend.into();
                    // MIDI pitch bend is 14-bit (0..16383), center is 8192
                    let norm = (raw as f32 - 8192.0) / 8192.0;
                    Some(ParsedMidiEvent::PitchBend {
                        channel: ch.index(),
                        bend_raw: raw,
                        bend_norm: norm.clamp(-1.0, 1.0),
                        timestamp_samples,
                    })
                }
                MidiMessage::PolyphonicKeyPressure(ch, note, pressure) => {
                    Some(ParsedMidiEvent::PolyphonicAftertouch {
                        channel: ch.index(),
                        note: note.into(),
                        pressure: pressure.into(),
                        timestamp_samples,
                    })
                }
                MidiMessage::ChannelPressure(ch, pressure) => {
                    Some(ParsedMidiEvent::ChannelAftertouch {
                        channel: ch.index(),
                        pressure: pressure.into(),
                        timestamp_samples,
                    })
                }
                _ => Some(ParsedMidiEvent::Ignored),
            },
            Err(_) => None,
        }
    }
}

/// Hardware MIDI subsystem backed by `midir`
pub struct HardwareMidiHost {
    client_name: String,
}

impl HardwareMidiHost {
    pub fn new(client_name: impl Into<String>) -> Self {
        Self {
            client_name: client_name.into(),
        }
    }

    /// Enumerate available physical and virtual MIDI input devices
    pub fn enumerate_inputs(&self) -> Result<Vec<MidiDeviceInfo>, String> {
        let midi_in = MidiInput::new(&self.client_name).map_err(|e| e.to_string())?;
        let ports = midi_in.ports();
        let mut devices = Vec::with_capacity(ports.len());

        for (id, port) in ports.iter().enumerate() {
            let name = midi_in
                .port_name(port)
                .unwrap_or_else(|_| format!("MIDI Port {}", id));
            devices.push(MidiDeviceInfo { id, name });
        }

        Ok(devices)
    }

    /// Connect to a MIDI input port with an isolated callback
    pub fn open_input_stream<F>(
        &self,
        port_index: usize,
        mut callback: F,
    ) -> Result<MidiInputConnection<()>, String>
    where
        F: FnMut(ParsedMidiEvent) + Send + 'static,
    {
        let mut midi_in = MidiInput::new(&self.client_name).map_err(|e| e.to_string())?;
        midi_in.ignore(Ignore::None);

        let ports = midi_in.ports();
        let port: &MidiInputPort = ports
            .get(port_index)
            .ok_or_else(|| format!("Invalid MIDI port index {}", port_index))?;

        let port_name = midi_in
            .port_name(port)
            .unwrap_or_else(|_| "Unknown Port".to_string());

        let connection = midi_in
            .connect(
                port,
                &port_name,
                move |timestamp_micros, data, _| {
                    // Approximate sample timestamp at 44.1kHz (44100 / 1_000_000 = 0.0441 samples/us)
                    let sample_timestamp = (timestamp_micros as f64 * 0.0441) as u64;
                    if let Some(event) = RealtimeMidiParser::parse_slice(data, sample_timestamp) {
                        callback(event);
                    }
                },
                (),
            )
            .map_err(|e| e.to_string())?;

        Ok(connection)
    }
}
