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

#[test]
fn test_mixer_stereo_separation_mid_side() {
    use Crystal_wave::mixing::channel_strip::ChannelStrip;

    let mut strip = ChannelStrip::new(0, "Stereo Test");
    let pan_law = (std::f32::consts::FRAC_PI_4).cos();

    // 1. Normal stereo (sigma = 0.0): Signal is unaltered
    strip.set_stereo_separation(0.0);
    let mut left = vec![1.0f32; 32];
    let mut right = vec![0.0f32; 32];
    strip.process_block(&mut left, &mut right, 1.0);
    assert!((left[0] - pan_law).abs() < 1e-4);
    assert!(right[0].abs() < 1e-4);

    // 2. 100% Merged (sigma = 1.0): Both channels become identical Mid sum: (L+R)/2 = 0.5
    strip.set_stereo_separation(1.0);
    let mut left = vec![1.0f32; 32];
    let mut right = vec![0.0f32; 32];
    strip.process_block(&mut left, &mut right, 1.0);
    assert!((left[0] - 0.5 * pan_law).abs() < 1e-4);
    assert!((right[0] - 0.5 * pan_law).abs() < 1e-4);

    // 3. 100% Separated (sigma = -1.0): Mid is zeroed, Side is amplified: L = L - R, R = R - L
    strip.set_stereo_separation(-1.0);
    let mut left = vec![1.0f32; 32];
    let mut right = vec![0.0f32; 32];
    strip.process_block(&mut left, &mut right, 1.0);
    assert!((left[0] - pan_law).abs() < 1e-4);
    assert!((right[0] - (-pan_law)).abs() < 1e-4);
}

#[test]
fn test_mixer_polarity_invert_and_channel_swap() {
    use Crystal_wave::mixing::channel_strip::ChannelStrip;

    let mut strip = ChannelStrip::new(0, "Phase/Swap Test");
    let pan_law = (std::f32::consts::FRAC_PI_4).cos();

    // Phase inversion: Left and Right are negated
    strip.set_phase_invert(true);
    let mut left = vec![0.75f32; 32];
    let mut right = vec![0.25f32; 32];
    strip.process_block(&mut left, &mut right, 1.0);
    assert!((left[0] - (-0.75 * pan_law)).abs() < 1e-4);
    assert!((right[0] - (-0.25 * pan_law)).abs() < 1e-4);

    // Channel Swap: Left and Right swap places
    strip.set_phase_invert(false);
    strip.set_swap_channels(true);
    let mut left = vec![0.8f32; 32];
    let mut right = vec![0.2f32; 32];
    strip.process_block(&mut left, &mut right, 1.0);
    assert!((left[0] - 0.2 * pan_law).abs() < 1e-4);
    assert!((right[0] - 0.8 * pan_law).abs() < 1e-4);
}

#[test]
fn test_mixer_10_insert_slots_bypass_and_wet_dry() {
    use Crystal_wave::mixing::channel_strip::{ChannelStrip, NUM_INSERT_SLOTS};

    let mut strip = ChannelStrip::new(1, "FX Strip");
    assert_eq!(strip.slots.len(), NUM_INSERT_SLOTS);
    assert_eq!(NUM_INSERT_SLOTS, 10);

    // Default slots are empty, unbypassed, 100% mix
    for (i, slot) in strip.slots.iter().enumerate() {
        assert_eq!(slot.slot_id, i);
        assert!(!slot.bypassed);
        assert_eq!(slot.mix, 1.0);
    }

    // Toggle slot bypass
    strip.set_slot_bypassed(3, true);
    assert!(strip.slots[3].bypassed);
    strip.set_slot_bypassed(3, false);
    assert!(!strip.slots[3].bypassed);

    // Adjust mix
    strip.set_slot_mix(2, 0.45);
    assert!((strip.slots[2].mix - 0.45).abs() < 1e-4);

    // Renaming
    strip.set_slot_name(0, "Fruity Limiter");
    assert_eq!(strip.slots[0].name, "Fruity Limiter");
}

#[test]
fn test_master_soft_clipper_saturation_and_ceiling() {
    use Crystal_wave::mixing::mixing_engine::{
        MASTER_CLIPPER_CEILING_DB, MASTER_CLIPPER_THRESH_DB, MixingEngine,
    };

    let ceiling_linear = 10.0f32.powf(MASTER_CLIPPER_CEILING_DB / 20.0);
    let thresh_linear = 10.0f32.powf(MASTER_CLIPPER_THRESH_DB / 20.0);

    // 1. Below threshold signal passes completely uncompressed (0% distortion)
    let mut left = vec![0.5f32; 64];
    let mut right = vec![-0.5f32; 64];
    MixingEngine::apply_soft_clipper(&mut left, &mut right);
    assert_eq!(left[0], 0.5);
    assert_eq!(right[0], -0.5);

    // 2. High overdrive signal saturates smoothly below or at -0.1 dBFS ceiling
    let mut left_hot = vec![3.0f32; 64];
    let mut right_hot = vec![-3.0f32; 64];
    MixingEngine::apply_soft_clipper(&mut left_hot, &mut right_hot);
    assert!(left_hot[0] > thresh_linear);
    assert!(left_hot[0] <= ceiling_linear + 1e-6);
    assert!(right_hot[0] < -thresh_linear);
    assert!(right_hot[0] >= -ceiling_linear - 1e-6);

    // 3. Extreme overs (+40 dBFS, amplitude = 100.0) strictly limited to ceiling
    let mut left_extreme = vec![100.0f32; 64];
    let mut right_extreme = vec![-100.0f32; 64];
    MixingEngine::apply_soft_clipper(&mut left_extreme, &mut right_extreme);
    assert!(left_extreme[0] <= ceiling_linear + 1e-6);
    assert!(right_extreme[0] >= -ceiling_linear - 1e-6);
}

#[test]
fn test_mixing_engine_125_insert_tracks_allocation() {
    use Crystal_wave::mixing::mixing_engine::{FL_STUDIO_MAX_INSERT_TRACKS, MixingEngine};

    let mut mixer = MixingEngine::new(44100.0).expect("Mixer init");
    assert_eq!(FL_STUDIO_MAX_INSERT_TRACKS, 125);

    // Allocate 125 tracks
    mixer.init_fl_insert_tracks(125).expect("Init 125 tracks");
    assert_eq!(mixer.channels.len(), 125);
    assert_eq!(mixer.channels[0].name, "Insert 1");
    assert_eq!(mixer.channels[124].name, "Insert 125");

    // Dynamic expansion via ensure_channel_exists
    mixer
        .ensure_channel_exists(150)
        .expect("Expand to 151 tracks");
    assert!(mixer.channels.len() >= 151);
}
