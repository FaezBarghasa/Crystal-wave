use Crystal_wave::timeline::*;

#[test]
fn test_track_alternatives_and_freeze() {
    let mut track = Track::new(0, "Lead Guitar", TrackType::Audio);
    assert_eq!(track.track_type, TrackType::Audio);

    // Initial region
    let r1 = Region::new(1, 0, "Take A", 0, 960 * 4, RegionKind::Audio {
        file_path: Some("take_a.wav".into()),
        sample_offset: 0,
        sample_length: 44100,
    });
    track.add_region(r1.clone());

    // Create a new alternative slot "Alt 2"
    let alt_id = track.new_empty_alternative("Alt 2");
    assert_eq!(alt_id, 1);
    assert_eq!(track.regions.len(), 0);

    // Modify active track regions in Alt 2
    let r2 = Region::new(2, 0, "Take B", 960, 960 * 2, RegionKind::Audio {
        file_path: Some("take_b.wav".into()),
        sample_offset: 0,
        sample_length: 22050,
    });
    track.add_region(r2);

    // Switch back to Alternative 0 (Alt 1 / original)
    assert!(track.switch_alternative(0));
    assert_eq!(track.regions.len(), 1);
    assert_eq!(track.regions[0].name, "Take A");

    // Switch to Alternative 1 (Alt 2)
    assert!(track.switch_alternative(1));
    assert_eq!(track.regions.len(), 1);
    assert_eq!(track.regions[0].name, "Take B");

    // Freeze track
    track.freeze(FreezeMode::PreFaderFreeze, Some(vec![0.1; 1000]));
    assert!(track.is_frozen);
    assert_eq!(track.freeze_mode, FreezeMode::PreFaderFreeze);
    assert_eq!(track.frozen_audio_buffer.as_ref().unwrap().len(), 1000);

    track.unfreeze();
    assert!(!track.is_frozen);
    assert!(track.frozen_audio_buffer.is_none());
}

#[test]
fn test_track_color_palette_and_contrast() {
    let col_dark = Color::new(20, 30, 40);
    assert!(col_dark.luminance() < 0.2);
    let text_col = col_dark.text_color();
    assert_eq!(text_col, Color::new(240, 240, 240)); // High contrast light text on dark bg

    let col_light = Color::new(240, 240, 240);
    assert!(col_light.luminance() > 0.8);
    assert_eq!(col_light.text_color(), Color::new(20, 20, 20)); // Dark text on light bg

    // Check 96 curated palette produces valid distinct colors
    let p0 = Color::curated_palette(0);
    let p24 = Color::curated_palette(24);
    assert_ne!(p0, p24);
}

#[test]
fn test_loop_region_punch_and_roll() {
    let mut loop_region = LoopRegion::new(960 * 4, 960 * 8);
    loop_region.set_punch(960 * 5, 960 * 7, true);
    loop_region.set_pre_post_roll(960, 960);

    assert_eq!(loop_region.length_ticks(), 960 * 4);
    assert!(!loop_region.is_in_punch_range(960 * 4)); // Before punch
    assert!(loop_region.is_in_punch_range(960 * 5));  // At punch in
    assert!(loop_region.is_in_punch_range(960 * 6));  // In punch
    assert!(!loop_region.is_in_punch_range(960 * 7)); // At punch out

    // Test loop wrap
    let wrapped = loop_region.wrap_tick(960 * 9);
    assert_eq!(wrapped, 960 * 5);
}

#[test]
fn test_region_anchor_offset_and_fades() {
    let mut r = Region::new(1, 0, "Vocal Pickup", 960 * 2, 960 * 4, RegionKind::Audio {
        file_path: None,
        sample_offset: 0,
        sample_length: 0,
    });
    r.set_anchor_offset(-240); // 1/16th note pickup before grid
    assert_eq!(r.effective_start_tick(), 960 * 2 - 240);

    r.set_fades(120, FadeCurve::EqualPower, 240, FadeCurve::Exponential);
    assert_eq!(r.fade.in_ticks, 120);
    assert_eq!(r.fade.in_curve, FadeCurve::EqualPower);
}

#[test]
fn test_relative_and_absolute_snapping() {
    let mode = SnappingMode::Beat; // 1 beat in 4/4 = 960 ticks
    let fps = 24.0;
    let bpm = 120.0;

    // Absolute snap to nearest beat
    let snapped_abs = mode.snap_with_style(1000, 4, 4, fps, bpm, SnapStyle::Absolute, 0);
    assert_eq!(snapped_abs, 960);

    // Relative snap with a 50-tick offset: grid points are 50, 1010, 1970, etc.
    let snapped_rel = mode.snap_with_style(1000, 4, 4, fps, bpm, SnapStyle::Relative, 50);
    assert_eq!(snapped_rel, 1010);
}

#[test]
fn test_scale_quantization() {
    let scale = MusicalScale::Dorian; // C Dorian: C(0), D(2), Eb(3), F(5), G(7), A(9), Bb(10)
    // Note E (MIDI 64 -> chroma 4, octave 5) should quantize to Eb (chroma 3 -> MIDI 63) or F (chroma 5 -> MIDI 65)
    let quantized_e = scale.quantize_pitch(64, 0);
    assert!(quantized_e == 63 || quantized_e == 65);

    // Note C (MIDI 60) should stay C (MIDI 60)
    assert_eq!(scale.quantize_pitch(60, 0), 60);

    // Minor Pentatonic: C(0), Eb(3), F(5), G(7), Bb(10)
    let pentatonic = MusicalScale::PentatonicMinor;
    assert_eq!(pentatonic.quantize_pitch(62, 0), 63); // D(62) -> Eb(63)
}

#[test]
fn test_arrangement_view_catch_playhead() {
    let timeline = Timeline::default();
    let mut view = ArrangementView::new(timeline);
    view.visible_start_tick = 0;
    view.visible_end_tick = 3840; // 1 bar (4 beats = 3840 ticks in default zoom)
    view.set_catch_playhead(CatchPlayheadMode::PageByPage);

    view.update_playhead_view(3840);
    // PageByPage should advance view
    assert_eq!(view.visible_start_tick, 3840);
    assert_eq!(view.visible_end_tick, 7680);
}
