use Crystal_wave::format::aaf_edl::{AafSession, EdlEvent, EdlTransition};
use Crystal_wave::format::timecode::{SmpteFrameRate, SmpteTimecode};

#[test]
fn test_smpte_timecode_parsing_and_conversions() {
    let tc = SmpteTimecode::parse("01:04:22:14", SmpteFrameRate::Fps25)
        .expect("Valid SMPTE timecode string");

    assert_eq!(tc.hours, 1);
    assert_eq!(tc.minutes, 4);
    assert_eq!(tc.seconds, 22);
    assert_eq!(tc.frames, 14);
    assert_eq!(tc.frame_rate, SmpteFrameRate::Fps25);

    // Test formatted output string
    assert_eq!(tc.to_string_formatted(), "01:04:22:14");

    // Test conversion to seconds
    let seconds = tc.to_seconds();
    let expected_seconds = (3600.0) + (4.0 * 60.0) + 22.0 + (14.0 / 25.0);
    assert!((seconds - expected_seconds).abs() < 1e-4);

    // Test conversion to ticks at 120 BPM
    let ticks = tc.to_ticks(120.0);
    assert!(ticks > 0);

    // Test error handling
    assert!(SmpteTimecode::parse("01:04", SmpteFrameRate::Fps25).is_err());
    assert!(SmpteTimecode::parse("invalid:tc:string:here", SmpteFrameRate::Fps25).is_err());
}

#[test]
fn test_aaf_session_and_edl_export() {
    let mut session = AafSession::new("Film_Reel_1", 48000.0, SmpteFrameRate::Fps24);
    assert_eq!(session.project_name, "Film_Reel_1");
    assert_eq!(session.sample_rate, 48000.0);

    let trk_idx = session.add_track("Dialogue_DX_01", true);
    assert_eq!(trk_idx, 0);
    assert_eq!(session.tracks.len(), 1);

    // Add EDL Event
    let src_in = SmpteTimecode::new(0, 0, 10, 0, SmpteFrameRate::Fps24);
    let src_out = SmpteTimecode::new(0, 0, 15, 0, SmpteFrameRate::Fps24);
    let master_in = SmpteTimecode::new(1, 0, 0, 0, SmpteFrameRate::Fps24);
    let master_out = SmpteTimecode::new(1, 0, 5, 0, SmpteFrameRate::Fps24);

    session.edl_events.push(EdlEvent {
        event_num: 1,
        reel_name: "TAPE01".to_string(),
        track_type: "A".to_string(),
        transition: EdlTransition::Cut,
        src_in,
        src_out,
        master_in,
        master_out,
    });

    let edl_output = session.export_cmx3600_edl();
    assert!(edl_output.contains("TITLE: Film_Reel_1"));
    assert!(edl_output.contains("FCM: NON-DROP FRAME"));
    assert!(edl_output.contains("001  TAPE01   A     C"));
    assert!(edl_output.contains("00:00:10:00 00:00:15:00 01:00:00:00 01:00:05:00"));
}
