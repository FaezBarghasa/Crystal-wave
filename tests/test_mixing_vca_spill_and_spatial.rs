use Crystal_wave::mixing::automation_preview::AutomationPreviewManager;
use Crystal_wave::mixing::spatial_audio_processor::{BED_CHANNELS_7_1_4, SpatialObjectPanner};

#[test]
fn test_automation_preview_audition_and_commit() {
    let mut preview_mgr = AutomationPreviewManager::new();
    assert!(!preview_mgr.is_preview_active);

    // Default automated value is 0.5
    let normal_val = preview_mgr.get_effective_value(0, "volume", 0.5);
    assert_eq!(normal_val, 0.5);

    // Auditioning while inactive does nothing
    preview_mgr.audition_param(0, "volume", 0.8);
    assert_eq!(preview_mgr.get_effective_value(0, "volume", 0.5), 0.5);

    // Engage preview mode
    preview_mgr.engage_preview();
    assert!(preview_mgr.is_preview_active);

    // Audition parameter change
    preview_mgr.audition_param(0, "volume", 0.85);
    assert_eq!(preview_mgr.get_effective_value(0, "volume", 0.5), 0.85);

    // Another param not auditioned retains automated value
    assert_eq!(preview_mgr.get_effective_value(0, "pan", 0.0), 0.0);

    // Commit write preview
    let committed = preview_mgr.commit_write_preview();
    assert_eq!(committed.len(), 1);
    assert_eq!(committed[0], (0, "volume".to_string(), 0.85));

    // After commit, preview is inactive
    assert!(!preview_mgr.is_preview_active);
    assert_eq!(preview_mgr.get_effective_value(0, "volume", 0.5), 0.5);
}

#[test]
fn test_spatial_object_panner_3d_rendering() {
    let mut panner = SpatialObjectPanner::new(0.0, 0.0, 0.0);
    panner.move_to(45.0, 30.0);
    assert_eq!(panner.azimuth_deg, 45.0);
    assert_eq!(panner.elevation_deg, 30.0);

    let gains = panner.calculate_bed_gains();
    assert_eq!(gains.len(), BED_CHANNELS_7_1_4);

    // Energy normalization check (sum of squares ~ 1.0)
    let sum_sq: f32 = gains.iter().map(|&g| g * g).sum();
    assert!((sum_sq - 1.0).abs() < 1e-4);

    // Test binaural downmix
    let (left, right) = panner.binaural_downmix(&gains);
    assert!(left > 0.0);
    assert!(right > 0.0);

    // Test 3D block rendering
    let mono_in = vec![1.0f32; 64];
    let mut ch0 = vec![0.0f32; 64];
    let mut ch1 = vec![0.0f32; 64];
    let mut ch2 = vec![0.0f32; 64];
    let mut ch3 = vec![0.0f32; 64];
    let mut ch4 = vec![0.0f32; 64];
    let mut ch5 = vec![0.0f32; 64];
    let mut ch6 = vec![0.0f32; 64];
    let mut ch7 = vec![0.0f32; 64];
    let mut ch8 = vec![0.0f32; 64];
    let mut ch9 = vec![0.0f32; 64];
    let mut ch10 = vec![0.0f32; 64];
    let mut ch11 = vec![0.0f32; 64];

    let mut channels_12: Vec<&mut [f32]> = vec![
        &mut ch0, &mut ch1, &mut ch2, &mut ch3, &mut ch4, &mut ch5, &mut ch6, &mut ch7, &mut ch8,
        &mut ch9, &mut ch10, &mut ch11,
    ];

    panner.render_3d_block(&mono_in, &mut channels_12);

    // Verify channel rendered signals reflect gains
    assert_eq!(channels_12[0][0], gains[0]);
    assert_eq!(channels_12[1][0], gains[1]);
}
