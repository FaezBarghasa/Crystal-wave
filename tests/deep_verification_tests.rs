use Crystal_wave::*;

#[test]
fn test_dolby_atmos_3d_bed_gains() {
    // 1. Center channel test (azimuth 0, elevation 0)
    let panner_center = DolbyAtmosPanner::new(0.0, 0.0, 0.0);
    let gains_center = panner_center.calculate_bed_gains();
    // Front Left, Right and Center should receive energy; heights should be 0
    assert!(gains_center[0] > 0.0);
    assert!(gains_center[1] > 0.0);
    assert!(gains_center[2] > 0.0);
    assert_eq!(gains_center[8], 0.0); // Top Front Left
    assert_eq!(gains_center[9], 0.0); // Top Front Right

    // 2. Pure Overhead Height test (elevation 90 deg)
    let panner_top = DolbyAtmosPanner::new(0.0, 90.0, 0.0);
    let gains_top = panner_top.calculate_bed_gains();
    // Front bed channels should be 0, overheads must receive energy
    assert_eq!(gains_top[0], 0.0);
    assert_eq!(gains_top[1], 0.0);
    assert!(gains_top[8] > 0.0);
    assert!(gains_top[9] > 0.0);
}

#[test]
fn test_auto_mute_protection_trip() {
    let sample_rate = 44100.0;
    let mut strip = ChannelStrip::new(1, "Vocal Track".into(), sample_rate);
    strip.compressor = None; // Test direct channel strip overload protection

    // Normal signal: 0 dBFS (amplitude 1.0) -> must not trip
    let mut left_normal = vec![1.0; 256];
    let mut right_normal = vec![1.0; 256];
    strip
        .process_block(&mut left_normal, &mut right_normal, 1.0)
        .unwrap();
    assert!(!strip.auto_mute_tripped);

    // Dangerously hot signal: +15 dBFS (amplitude 10.0, above +12 dBFS / 3.98 limit)
    let mut left_blast = vec![10.0; 256];
    let mut right_blast = vec![10.0; 256];
    strip
        .process_block(&mut left_blast, &mut right_blast, 1.0)
        .unwrap();

    // Auto-Mute safety mechanism must trip and mute output to 0.0
    assert!(strip.auto_mute_tripped);
    assert!(left_blast.iter().all(|&s| s == 0.0));
    assert!(right_blast.iter().all(|&s| s == 0.0));
}

#[test]
fn test_dual_channel_strip_inspector_routing() {
    let mut inspector = DualChannelInspector::new(1, 0); // Track 1 -> Master Out (Bus 0)
    assert_eq!(inspector.selected_track_id, 1);
    assert_eq!(inspector.downstream_destination_bus, 0);

    // Track 2 routes to Drum Subgroup (Aux Bus 5)
    inspector.update_selection(2, 5);
    assert_eq!(inspector.selected_track_id, 2);
    assert_eq!(inspector.downstream_destination_bus, 5);
}

#[test]
fn test_smart_controls_macro_mapping_and_inversion() {
    let mut knob = SmartKnob::new("Cutoff / Drive Macro".into());

    // Mapping 0: Filter Cutoff (200 Hz to 10000 Hz, normal)
    knob.add_mapping(ParameterMapping {
        target_plugin_slot: 0,
        parameter_id: 1,
        min_value: 200.0,
        max_value: 10000.0,
        invert: false,
    });

    // Mapping 1: Clean Dry Blend (1.0 to 0.0, inverted)
    knob.add_mapping(ParameterMapping {
        target_plugin_slot: 1,
        parameter_id: 2,
        min_value: 0.0,
        max_value: 1.0,
        invert: true,
    });

    // At knob = 0.0
    knob.set_value(0.0);
    assert_eq!(knob.evaluate_mapping(0).unwrap(), 200.0);
    assert_eq!(knob.evaluate_mapping(1).unwrap(), 1.0); // Inverted max

    // At knob = 0.5
    knob.set_value(0.5);
    assert_eq!(knob.evaluate_mapping(0).unwrap(), 5100.0);
    assert_eq!(knob.evaluate_mapping(1).unwrap(), 0.5);

    // At knob = 1.0
    knob.set_value(1.0);
    assert_eq!(knob.evaluate_mapping(0).unwrap(), 10000.0);
    assert_eq!(knob.evaluate_mapping(1).unwrap(), 0.0);
}

#[test]
fn test_timeline_tick_to_seconds_with_tempo_changes() {
    let mut timeline = Timeline::default();
    // Bar 1 (0 to 3840 ticks) at 120 BPM: 4 beats = 2.0 seconds
    timeline.tempo_map = vec![
        crate::timeline::TempoEvent {
            tick: 0,
            bpm: 120.0,
        },
        // Bar 2 (3840 ticks) jumps to 60 BPM (2x slower)
        crate::timeline::TempoEvent {
            tick: 3840,
            bpm: 60.0,
        },
    ];

    // Tick 3840 (end of bar 1) should be exactly 2.0 seconds
    let sec_bar1 = timeline.tick_to_seconds(3840);
    assert!((sec_bar1 - 2.0).abs() < 1e-4);

    // Tick 7680 (end of bar 2) should add 4 beats at 60 BPM (= 4.0 seconds) -> Total 6.0 seconds
    let sec_bar2 = timeline.tick_to_seconds(7680);
    assert!((sec_bar2 - 6.0).abs() < 1e-4);
}
