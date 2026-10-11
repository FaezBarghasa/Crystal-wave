use Crystal_wave::pro_tools::{IoMatrix, PortDirection, VcaManager};

#[test]
fn test_vca_master_gain_scaling_and_mutes() {
    let mut vca_mgr = VcaManager::new();

    let drum_vca = vca_mgr.create_vca("Drums VCA");
    let all_vca = vca_mgr.create_vca("All Music VCA");

    // Assign tracks 0, 1, 2 to Drums VCA
    vca_mgr.assign_track_to_vca(drum_vca, 0).unwrap();
    vca_mgr.assign_track_to_vca(drum_vca, 1).unwrap();
    vca_mgr.assign_track_to_vca(drum_vca, 2).unwrap();

    // Assign Drums VCA and Vocal track 3 to All Music VCA
    vca_mgr.assign_track_to_vca(all_vca, 0).unwrap();
    vca_mgr.assign_track_to_vca(all_vca, 1).unwrap();
    vca_mgr.assign_track_to_vca(all_vca, 2).unwrap();
    vca_mgr.assign_track_to_vca(all_vca, 3).unwrap();

    // 1. Nominal gain (0 dB everywhere) -> Multiplier 1.0
    let gain_nominal = vca_mgr.calculate_effective_linear_gain(0, 0.0);
    assert!((gain_nominal - 1.0).abs() < 1e-4);

    // 2. Reduce Drums VCA by -6 dB, track at 0 dB -> Multiplier ~0.5012
    vca_mgr.set_vca_gain_db(drum_vca, -6.0).unwrap();
    let gain_drums = vca_mgr.calculate_effective_linear_gain(0, 0.0);
    assert!((gain_drums - 0.5011872).abs() < 1e-3);

    // Vocal track 3 (not in Drums VCA, only All Music VCA) remains unaffected
    let gain_vocal = vca_mgr.calculate_effective_linear_gain(3, 0.0);
    assert!((gain_vocal - 1.0).abs() < 1e-4);

    // 3. Nested VCA: All Music VCA pulled down by -4 dB (-6 + -4 = -10 dB total on drums)
    vca_mgr.set_vca_gain_db(all_vca, -4.0).unwrap();
    let gain_nested_drums = vca_mgr.calculate_effective_linear_gain(0, 0.0);
    assert!((gain_nested_drums - 0.3162277).abs() < 1e-3); // -10 dB

    // 4. Mute ganging
    assert!(!vca_mgr.is_track_muted(0, false));
    vca_mgr.set_vca_mute(drum_vca, true).unwrap();
    assert!(vca_mgr.is_track_muted(0, false));
    assert!(vca_mgr.is_track_muted(1, false));
    assert!(!vca_mgr.is_track_muted(3, false)); // Vocal track not muted

    // 5. Solo ganging
    assert!(!vca_mgr.is_track_soloed(3, false));
    vca_mgr.set_vca_solo(all_vca, true).unwrap();
    assert!(vca_mgr.is_track_soloed(3, false));
}

#[test]
fn test_hardware_insert_io_routing_and_ping_calibration() {
    let mut matrix = IoMatrix::new();

    // Physical ports on audio interface:
    let out_3 = matrix.add_port("Line Out 3 (Compressor Send)", PortDirection::Output, 2);
    let in_3 = matrix.add_port("Line In 3 (Compressor Return)", PortDirection::Input, 2);

    let insert_id = matrix
        .create_hardware_insert("Analog 1176 Insert", out_3, in_3)
        .expect("Create insert");

    // Ping latency measurement:
    // Generate 128-sample test buffer with a 1.0 impulse at sample 10
    let mut send_impulse = vec![0.0f32; 128];
    send_impulse[10] = 1.0;

    // Simulate loopback signal returned through DA/AD converters with 42 samples round-trip latency
    let mut received_signal = vec![0.0f32; 128];
    received_signal[10 + 42] = 0.95; // returned impulse at sample 52

    let measured_delay = IoMatrix::calculate_ping_latency(&send_impulse, &received_signal, 0.5)
        .expect("Ping detection");

    assert_eq!(measured_delay, 42);

    matrix
        .calibrate_insert_delay(insert_id, measured_delay)
        .expect("Calibrate insert");

    assert_eq!(
        matrix.hardware_inserts[insert_id].delay_compensation_samples,
        42
    );
}
