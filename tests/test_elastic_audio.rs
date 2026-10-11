use Crystal_wave::elastic::*;

#[test]
fn test_elastic_audio_time_stretch_modes() {
    let sample_rate = 44100.0;
    let input = vec![0.5f32; 1000];
    let mut output = vec![0.0f32; 2000]; // 2x stretch

    // 1. Speed mode
    let mut elastic_speed = ElasticAudio::new(FlexTimeMode::Speed, sample_rate).unwrap();
    elastic_speed.add_warp_marker(0, 0);
    elastic_speed.add_warp_marker(1000, 2000);
    elastic_speed.process(&input, &mut output, 2.0).unwrap();
    assert!((output[1000] - 0.5).abs() < 1e-3);

    // 2. Monophonic mode
    let elastic_mono = ElasticAudio::new(FlexTimeMode::Monophonic, sample_rate).unwrap();
    let mut out_mono = vec![0.0f32; 2000];
    elastic_mono.process(&input, &mut out_mono, 2.0).unwrap();
    assert!(out_mono.iter().any(|&s| s > 0.0));

    // 3. Rhythmic mode
    let elastic_rhythm = ElasticAudio::new(FlexTimeMode::Rhythmic, sample_rate).unwrap();
    let mut out_rhythm = vec![0.0f32; 2000];
    elastic_rhythm
        .process(&input, &mut out_rhythm, 2.0)
        .unwrap();
    assert!(out_rhythm.iter().any(|&s| s > 0.0));
}

#[test]
fn test_flex_pitch_6_hotspot_engine() {
    let sample_rate = 44100.0;
    let engine = FlexPitchEngine::new(sample_rate);

    let note = FlexPitch6Hotspot {
        start_sample: 0,
        length_samples: 44100, // 1 second note
        original_pitch_midi: 60.0,
        pitch_drift_in_cents: 50.0,   // +50 cents at start
        fine_pitch_cents: 10.0,       // +10 cents overall
        pitch_drift_out_cents: -30.0, // -30 cents at end
        gain_db: 3.0,                 // +3 dB
        vibrato_depth_percent: 20.0,  // vibrato
        formant_shift_semitones: 2.0, // +2 semitones
    };

    // At start (sample 0): fine (10) + drift_in (50) = 60 cents = 0.6 semitones
    let offset_start = engine.calculate_instantaneous_pitch_offset(&note, 0);
    assert!((offset_start - 0.6).abs() < 1e-2);

    // At mid note (sample 22050): drift_in and drift_out are 0
    let offset_mid = engine.calculate_instantaneous_pitch_offset(&note, 22050);
    // Vibrato fluctuates around fine (10 cents = 0.1 semitones)
    assert!((offset_mid - 0.1).abs() < 0.2);

    // Gain processing test
    let mut buffer = vec![1.0f32; 44100];
    engine.process_note(&note, &mut buffer);
    let expected_gain = 10.0f32.powf(3.0 / 20.0);
    assert!((buffer[1000] - expected_gain).abs() < 1e-4);
}
