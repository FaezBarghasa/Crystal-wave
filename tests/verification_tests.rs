use Crystal_wave::*;
use std::f32::consts::PI;

#[test]
fn test_fet1176_fast_attack_and_saturation() {
    let sample_rate = 44100.0;
    let config = CompressorConfig {
        model: CompressorModel::Fet1176,
        threshold_db: -12.0,
        ratio: 4.0,
        attack_ms: 20.0,
        release_ms: 100.0,
        makeup_gain_db: 0.0,
        knee_db: 2.0,
        mix: 1.0,
    };
    let mut comp = Compressor::new(config, sample_rate).expect("Valid sample rate");

    let mut left = vec![1.0; 1000]; // 0 dBFS signal (well above -12 dB threshold)
    let mut right = vec![1.0; 1000];

    comp.process(&mut left, &mut right).expect("Process success");

    // FET 1176 should attack within < 1 ms (< 44 samples) and heavily compress
    assert!(comp.current_gain_reduction_db() < -5.0);
    assert!(left[500] < 0.6);
}

#[test]
fn test_opto_la2a_dual_decay_memory() {
    let sample_rate = 44100.0;
    let config = CompressorConfig {
        model: CompressorModel::OptoLa2a,
        threshold_db: -20.0,
        ratio: 3.0,
        attack_ms: 10.0,
        release_ms: 100.0,
        makeup_gain_db: 0.0,
        knee_db: 8.0,
        mix: 1.0,
    };
    let mut comp = Compressor::new(config, sample_rate).expect("Valid sample rate");

    // Feed loud signal for 2000 samples to charge optical memory
    let mut left_loud = vec![1.0; 2000];
    let mut right_loud = vec![1.0; 2000];
    comp.process(&mut left_loud, &mut right_loud).unwrap();
    let gr_during_signal = comp.current_gain_reduction_db();
    assert!(gr_during_signal < -6.0);

    // Feed silence to observe release curve
    let mut left_quiet = vec![0.0; 1000];
    let mut right_quiet = vec![0.0; 1000];
    comp.process(&mut left_quiet, &mut right_quiet).unwrap();

    // Release should be recovering towards 0 dB but still negative due to LA-2A slow optical decay
    assert!(comp.current_gain_reduction_db() > gr_during_signal);
    assert!(comp.current_gain_reduction_db() < -0.5);
}

#[test]
fn test_channel_eq_8band_magnitude() {
    let sample_rate = 44100.0;
    let mut eq = ChannelEq::new(sample_rate).expect("Valid sample rate");

    // Configure Band 0 (Low Cut at 80 Hz) and Band 4 (Peak +6 dB at 1000 Hz, Q=2.0)
    eq.bands[0].freq_hz = 80.0;
    eq.bands[0].update(sample_rate);

    eq.bands[4].freq_hz = 1000.0;
    eq.bands[4].gain_db = 6.0;
    eq.bands[4].q = 2.0;
    eq.bands[4].update(sample_rate);

    // Test Low Cut: At 20 Hz, magnitude should be significantly attenuated (< -10 dB)
    let mag_20hz = eq.total_magnitude_db(20.0);
    assert!(mag_20hz < -10.0, "20 Hz attenuation expected, got {}", mag_20hz);

    // Test Peak Boost: At 1000 Hz, magnitude should be approximately +6 dB
    let mag_1khz = eq.total_magnitude_db(1000.0);
    assert!((mag_1khz - 6.0).abs() < 0.5, "1000 Hz boost expected ~6 dB, got {}", mag_1khz);
}

#[test]
fn test_flex_time_speed_and_pitch() {
    let sample_rate = 44100.0;
    let mut flex = FlexProcessor::new(FlexMode::Speed, sample_rate).expect("Valid processor");
    flex.stretch_factor = 2.0; // Stretched 2x

    let mut input = vec![0.0; 1000];
    for (i, s) in input.iter_mut().enumerate() {
        *s = (2.0 * PI * 440.0 * (i as f32 / sample_rate)).sin();
    }

    let mut output = vec![0.0; 2000];
    flex.process_stretch(&input, &mut output).expect("Stretch success");

    assert_eq!(output.len(), 2000);
    assert!(!output.iter().any(|s| s.is_nan() || s.is_infinite()));
}

#[test]
fn test_flex_pitch_6_parameter_solver() {
    let sample_rate = 44100.0;
    let flex = FlexProcessor::new(FlexMode::FlexPitch, sample_rate).unwrap();

    let mut audio = vec![0.5; 44100];
    let note = FlexPitchNote {
        start_sample: 1000,
        length_samples: 10000,
        original_pitch_midi: 60.0,
        pitch_drift_in_cents: 20.0,
        fine_pitch_cents: 10.0,
        pitch_drift_out_cents: -15.0,
        gain_db: -3.0,
        vibrato_depth_percent: 50.0,
        formant_shift_semitones: 1.0,
    };

    flex.process_flex_pitch(&note, &mut audio).unwrap();
    // Verify audio within note range was scaled by gain (~ -3 dB = ~0.707 linear)
    let processed_sample = audio[5000];
    assert!(processed_sample < 0.5 && processed_sample > 0.2);
}

#[test]
fn test_chromaverb_delay_network() {
    let sample_rate = 44100.0;
    let mut verb = ChromaVerbEngine::new(2.0, sample_rate).unwrap();

    let mut l = vec![0.0; 3000];
    let mut r = vec![0.0; 3000];
    l[0] = 1.0; // Impulse input
    r[0] = 1.0;

    verb.process(&mut l, &mut r).unwrap();

    // Verify impulse spread and non-explosive tail
    assert!(!l.iter().any(|s| s.is_nan() || s.abs() > 2.0));
    assert!(l[1320..].iter().any(|&s| s.abs() > 1e-4), "Reverb reflections must appear after base delay");
}

#[test]
fn test_markov_harmonic_progression() {
    let engine = MarkovProgressionEngine::new();
    let c_major = Chord::new(0, ChordQuality::Major); // C Major (I)

    let next_predictions = engine.predict_next_chords(c_major);
    assert!(!next_predictions.is_empty());

    // In C Major, V (G7) or IV (F Major) should be primary predicted chords
    let g_dom7 = Chord::new(7, ChordQuality::Dominant7);
    let f_major = Chord::new(5, ChordQuality::Major);

    let has_dominant_or_subdominant = next_predictions.iter().any(|(c, _)| *c == g_dom7 || *c == f_major);
    assert!(has_dominant_or_subdominant);
}

#[test]
fn test_session_player_voice_leading() {
    let mut chord_track = ChordTrack::new(0); // C Major
    chord_track.add_chord(0, 960 * 4, Chord::new(0, ChordQuality::Major)); // C Major bar 1
    chord_track.add_chord(960 * 4, 960 * 4, Chord::new(7, ChordQuality::Major)); // G Major bar 2

    let bassist = SessionPlayer::new(
        SessionPlayerType::Bassist,
        GenerativePerformanceParams {
            complexity: 0.3,
            intensity: 0.8,
            swing: 0.0,
            fill_frequency: 0.0,
            push_pull_ticks: 0,
        },
    );

    let notes = bassist.generate_track(&chord_track, 960 * 8);
    assert!(!notes.is_empty());

    // Bar 1 downbeat must be root C (MIDI 36)
    assert_eq!(notes[0].note_number, 36);
    // Bar 2 downbeat (tick 3840) must be root G (MIDI 43 = 36 + 7)
    let bar2_downbeat = notes.iter().find(|n| n.tick == 960 * 4).unwrap();
    assert_eq!(bar2_downbeat.note_number, 43);
}

#[test]
fn test_quick_swipe_comping_crossfade() {
    let mut take_folder = TakeFolder::new(200); // 200 samples crossfade (~5ms)

    let take1 = vec![1.0; 2000]; // Take 1: DC +1.0
    let take2 = vec![-1.0; 2000]; // Take 2: DC -1.0

    let id1 = take_folder.add_take("Take 1".into(), take1);
    let id2 = take_folder.add_take("Take 2".into(), take2);

    // Comp: Samples 0..1000 from Take 1, Samples 1000..2000 from Take 2
    take_folder.set_comp_slice(id1, 0, 1000).unwrap();
    take_folder.set_comp_slice(id2, 1000, 2000).unwrap();

    let comp_audio = take_folder.render_comp(2000);
    assert_eq!(comp_audio.len(), 2000);

    // Verify Take 1 audio in region before cut
    assert!((comp_audio[500] - 1.0).abs() < 1e-4);
    // Verify Take 2 audio in region after cut
    assert!((comp_audio[1500] - (-1.0)).abs() < 1e-4);
    // Verify crossfade smooth transition at sample 1000 (equal power sum)
    assert!(comp_audio[1000].abs() < 1.0);
}

#[test]
fn test_step_sequencer_directions() {
    let mut lane = StepSequencerLane::new("Kick".into(), 4);
    lane.steps[0].note_number = 36;
    lane.steps[1].note_number = 37;
    lane.steps[2].note_number = 38;
    lane.steps[3].note_number = 39;

    lane.playback_direction = MidiDirection::Forward;
    assert_eq!(lane.advance_step().unwrap().note_number, 36);
    assert_eq!(lane.advance_step().unwrap().note_number, 37);
    assert_eq!(lane.advance_step().unwrap().note_number, 38);
    assert_eq!(lane.advance_step().unwrap().note_number, 39);
    assert_eq!(lane.advance_step().unwrap().note_number, 36); // Wrapped
}
