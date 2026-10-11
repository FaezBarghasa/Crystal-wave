use Crystal_wave::audio::StemSeparatorEngine;
use Crystal_wave::audio_edit::{DecoupledPlayheadSelection, PlaybackTarget, TapeScrubEngine};

#[test]
fn test_decoupled_playhead_and_selection_loop() {
    let mut decoupled = DecoupledPlayheadSelection::new();

    // 1. Park arrangement playhead at verse start (tick 960 * 4)
    decoupled.park_playhead_at(960 * 4);
    assert_eq!(decoupled.parked_playhead_tick, 960 * 4);
    assert_eq!(decoupled.current_playhead_tick, 960 * 4);

    // 2. Highlight microscopic 5ms vocal consonant (ticks 12000..12050)
    decoupled.set_edit_selection(1, 12000, 12050);
    assert!(decoupled.is_selection_active);

    // 3. Engage selection loop
    decoupled
        .engage_selection_loop()
        .expect("Engage selection loop");
    assert_eq!(decoupled.playback_target, PlaybackTarget::EditSelectionLoop);
    assert_eq!(decoupled.current_playhead_tick, 12000);

    // 4. Advance playback inside the selection loop
    // Advancing 30 ticks should be at 12030
    let pos1 = decoupled.advance_playback(30);
    assert_eq!(pos1, 12030);

    // Advancing another 30 ticks wraps within the 50-tick loop (12000 + 10 = 12010)
    let pos2 = decoupled.advance_playback(30);
    assert_eq!(pos2, 12010);

    // Crucial Invariant: The main arrangement playhead stayed parked the entire time!
    assert_eq!(decoupled.parked_playhead_tick, 960 * 4);

    // 5. Restore arrangement playhead
    decoupled.restore_parked_playhead();
    assert_eq!(decoupled.playback_target, PlaybackTarget::ArrangementTimeline);
    assert_eq!(decoupled.current_playhead_tick, 960 * 4);
}

#[test]
fn test_tape_style_audio_scrubbing() {
    let sample_rate = 44100.0f32;
    let mut scrub_engine = TapeScrubEngine::new(sample_rate, 500.0); // 500 px/sec

    // 1. Forward mouse drag (250 px in 0.5s = 500 px/sec -> exactly 1.0x speed)
    let speed_fwd = scrub_engine.calculate_scrub_speed(250.0, 0.5);
    assert!((speed_fwd - 1.0).abs() < 1e-3);

    // 2. Fast reverse mouse drag (-500 px in 0.5s = -1000 px/sec -> -2.0x speed)
    let speed_rev = scrub_engine.calculate_scrub_speed(-500.0, 0.5);
    assert!((speed_rev - (-2.0)).abs() < 1e-3);

    // 3. Render scrub block across a sine waveform
    let test_audio: Vec<f32> = (0..1000)
        .map(|i| ((i as f32) / 100.0).sin())
        .collect();
    let mut out_block = vec![0.0f32; 128];

    scrub_engine.current_sample_pos = 100.0;
    let rendered_speed = scrub_engine.render_scrub_block(&test_audio, 250.0, 0.5, &mut out_block);

    assert!((rendered_speed - 1.0).abs() < 1e-3);
    assert!(scrub_engine.current_sample_pos > 100.0);
    assert!(!out_block.iter().all(|&s| s == 0.0));
}

#[test]
fn test_native_4_stem_separation_and_phase_reconstruction() {
    let engine = StemSeparatorEngine::new();
    let num_samples = 4096;

    // Generate complex mixed track:
    // Low sub-bass (80 Hz) + Vocals in center (1 kHz) + Stereo instruments (4 kHz)
    let mut input_l = vec![0.0f32; num_samples];
    let mut input_r = vec![0.0f32; num_samples];

    for i in 0..num_samples {
        let t = i as f32 / 44100.0;
        let bass = (2.0 * std::f32::consts::PI * 80.0 * t).sin() * 0.6;
        let vocals = (2.0 * std::f32::consts::PI * 1000.0 * t).sin() * 0.4;
        let synth_l = (2.0 * std::f32::consts::PI * 4000.0 * t).sin() * 0.3;
        let synth_r = (2.0 * std::f32::consts::PI * 4000.0 * t).cos() * 0.3;

        input_l[i] = bass + vocals + synth_l;
        input_r[i] = bass + vocals + synth_r;
    }

    let stems = engine
        .separate_stereo(&input_l, &input_r)
        .expect("Stem separation succeeded");

    // Verify all 4 stems were produced with proper channel lengths
    assert_eq!(stems.vocals_l.len(), num_samples);
    assert_eq!(stems.drums_l.len(), num_samples);
    assert_eq!(stems.bass_l.len(), num_samples);
    assert_eq!(stems.other_l.len(), num_samples);

    // Crucial Phase Invariant:
    // The sum of all 4 stems must perfectly reconstruct the original input
    for i in 0..num_samples {
        let reconstructed_l = stems.bass_l[i] + stems.drums_l[i] + stems.vocals_l[i] + stems.other_l[i];
        let reconstructed_r = stems.bass_r[i] + stems.drums_r[i] + stems.vocals_r[i] + stems.other_r[i];

        assert!(
            (reconstructed_l - input_l[i]).abs() < 1e-4,
            "Reconstruction error on left channel at sample {}",
            i
        );
        assert!(
            (reconstructed_r - input_r[i]).abs() < 1e-4,
            "Reconstruction error on right channel at sample {}",
            i
        );
    }
}
