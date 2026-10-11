use Crystal_wave::midi::hardware_midi::{HardwareMidiHost, ParsedMidiEvent, RealtimeMidiParser};

#[test]
fn test_realtime_midi_parser_note_on_off() {
    // Note On: Channel 1 (0x90), Note 60 (Middle C = 0x3C), Velocity 100 (0x64)
    let note_on_bytes = [0x90, 0x3C, 0x64];
    let event = RealtimeMidiParser::parse_slice(&note_on_bytes, 44100);
    assert_eq!(
        event,
        Some(ParsedMidiEvent::NoteOn {
            channel: 0,
            note: 60,
            velocity: 100,
            timestamp_samples: 44100,
        })
    );

    // Note On with velocity 0 is treated as Note Off in MIDI specification
    let note_on_zero_vel = [0x90, 0x3C, 0x00];
    let event_zero = RealtimeMidiParser::parse_slice(&note_on_zero_vel, 44200);
    assert_eq!(
        event_zero,
        Some(ParsedMidiEvent::NoteOff {
            channel: 0,
            note: 60,
            velocity: 0,
            timestamp_samples: 44200,
        })
    );

    // Explicit Note Off: Channel 1 (0x80), Note 60, Velocity 64 (0x40)
    let note_off_bytes = [0x80, 0x3C, 0x40];
    let event_off = RealtimeMidiParser::parse_slice(&note_off_bytes, 44300);
    assert_eq!(
        event_off,
        Some(ParsedMidiEvent::NoteOff {
            channel: 0,
            note: 60,
            velocity: 64,
            timestamp_samples: 44300,
        })
    );
}

#[test]
fn test_realtime_midi_parser_cc_and_pitch_bend() {
    // Control Change: Channel 2 (0xB1), CC 7 (Volume = 0x07), Value 127 (0x7F)
    let cc_bytes = [0xB1, 0x07, 0x7F];
    let event_cc = RealtimeMidiParser::parse_slice(&cc_bytes, 88200);
    assert_eq!(
        event_cc,
        Some(ParsedMidiEvent::ControlChange {
            channel: 1,
            controller: 7,
            value: 127,
            timestamp_samples: 88200,
        })
    );

    // Pitch Bend Center: Channel 1 (0xE0), LSB 0x00, MSB 0x40 (Value 8192)
    let bend_center = [0xE0, 0x00, 0x40];
    let event_center = RealtimeMidiParser::parse_slice(&bend_center, 88300);
    match event_center {
        Some(ParsedMidiEvent::PitchBend {
            channel,
            bend_raw,
            bend_norm,
            ..
        }) => {
            assert_eq!(channel, 0);
            assert_eq!(bend_raw, 8192);
            assert!((bend_norm - 0.0).abs() < 0.001);
        }
        other => panic!("Unexpected event: {:?}", other),
    }

    // Pitch Bend Max: Channel 1 (0xE0), LSB 0x7F, MSB 0x7F (Value 16383)
    let bend_max = [0xE0, 0x7F, 0x7F];
    let event_max = RealtimeMidiParser::parse_slice(&bend_max, 88400);
    match event_max {
        Some(ParsedMidiEvent::PitchBend {
            bend_raw,
            bend_norm,
            ..
        }) => {
            assert_eq!(bend_raw, 16383);
            assert!((bend_norm - 1.0).abs() < 0.01);
        }
        other => panic!("Unexpected event: {:?}", other),
    }
}

#[test]
fn test_hardware_midi_host_device_enumeration() {
    let host = HardwareMidiHost::new("CrystalWaveTestClient");
    let inputs = host.enumerate_inputs();
    // In headless Linux CI, ALSA/JACK virtual MIDI ports may or may not exist, but enumeration must succeed cleanly without panic
    assert!(inputs.is_ok());
}
