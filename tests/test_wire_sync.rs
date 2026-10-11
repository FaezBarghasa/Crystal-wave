use Crystal_wave::{CrystaSessionEnvelope, EdgeRayAudioPacket, EdgeRayTelemetryDatagram};

#[test]
fn test_edgeray_audio_packet_roundtrip() {
    let samples = vec![0.1f32, -0.2f32, 0.5f32, -0.5f32, 0.0f32, 0.9f32];
    let packet = EdgeRayAudioPacket::new(42, 1_000_000_000, 48000, 2, &samples);

    assert_eq!(packet.sequence_number, 42);
    assert_eq!(packet.sample_rate, 48000);
    assert_eq!(packet.channels, 2);
    assert_eq!(packet.frame_count, 3);
    assert_eq!(packet.payload_f32, samples);

    let bytes = packet.to_bytes().expect("Serialization failed");
    let decoded = EdgeRayAudioPacket::from_bytes(&bytes).expect("Deserialization failed");

    assert_eq!(packet, decoded);
}

#[test]
fn test_edgeray_telemetry_datagram_roundtrip() {
    let datagram = EdgeRayTelemetryDatagram {
        session_id: "session-xyz-123".into(),
        playhead_tick: 3840,
        bpm: 124.5,
        is_playing: true,
        cpu_load_percent: 14.2,
        master_peak_db: -3.1,
        master_rms_db: -18.4,
    };

    let bytes = datagram.to_bytes().expect("Telemetry serialization");
    let decoded = EdgeRayTelemetryDatagram::from_bytes(&bytes).expect("Telemetry deserialization");

    assert_eq!(datagram, decoded);
}

#[test]
fn test_crysta_session_envelope_roundtrip() {
    let envelope = CrystaSessionEnvelope::new(
        "sess-uuid-777",
        "Atmos Master Session",
        "faez@edge.io",
        96000.0,
        130.0,
        64,
        r#"{"tracks":[{"id":0,"name":"Drums"}]}"#,
    );

    assert_eq!(envelope.session_id, "sess-uuid-777");
    assert_eq!(envelope.tracks_count, 64);
    assert_eq!(envelope.sample_rate, 96000.0);

    let bytes = envelope.to_bytes().expect("Envelope serialization");
    let decoded = CrystaSessionEnvelope::from_bytes(&bytes).expect("Envelope deserialization");

    assert_eq!(envelope, decoded);
}
