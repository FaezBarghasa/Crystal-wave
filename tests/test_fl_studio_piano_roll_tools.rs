use Crystal_wave::audio::playback_engine::TimelineAudioScheduler;
use Crystal_wave::fl_studio::piano_roll::{
    ArpDirection, FlPianoRollNote, FlPianoRollTools, MusicalScale,
};
use Crystal_wave::timeline::track::{Track, TrackType};
use Crystal_wave::ui::app::CrystalWaveApp;
use Crystal_wave::ui::message::UiMessage;

#[test]
fn test_piano_roll_articulation_tools_execution() {
    let mut app = CrystalWaveApp::new(44100.0);

    // Initial chord: 5 notes (C4, E4, G4, B4, C5 Slide)
    assert_eq!(app.piano_roll_notes.len(), 5);
    let original_c4_start = app.piano_roll_notes[0].start_tick;
    let original_b4_start = app.piano_roll_notes[3].start_tick;
    assert_eq!(original_c4_start, 0);
    assert_eq!(original_b4_start, 0);

    // 1. Quick Strum Tool: Staggers chord note start times
    app.update(UiMessage::PianoRollQuickStrum);
    let strummed_b4_start = app.piano_roll_notes[3].start_tick;
    assert!(
        strummed_b4_start > 0,
        "Strum tool must stagger start offsets"
    );

    // 2. Flip Vertical Tool: Inverts melodic intervals around key 64 (E4)
    // Key 60 (C4) -> inverted diff: 64 - (60 - 64) = 68 (G#4)
    let initial_key_0 = app.piano_roll_notes[0].key;
    app.update(UiMessage::PianoRollFlipVertical);
    let inverted_key_0 = app.piano_roll_notes[0].key;
    assert_ne!(initial_key_0, inverted_key_0);

    // 3. Quick Chop Tool: Subdivides notes into 1/16 rhythmic rolls
    let count_before_chop = app.piano_roll_notes.len();
    app.update(UiMessage::PianoRollQuickChop);
    assert_eq!(app.piano_roll_notes.len(), count_before_chop * 2);
}

#[test]
fn test_musical_scales_and_highlighting() {
    let major = MusicalScale::Major;
    let minor = MusicalScale::Minor;
    let dorian = MusicalScale::Dorian;
    let pentatonic = MusicalScale::PentatonicMajor;

    // C Major: C(60), D(62), E(64), F(65), G(67), A(69), B(71)
    assert!(major.is_in_scale(60, 60)); // C
    assert!(major.is_in_scale(60, 64)); // E
    assert!(major.is_in_scale(60, 67)); // G
    assert!(!major.is_in_scale(60, 61)); // C# / Db
    assert!(!major.is_in_scale(60, 63)); // Eb

    // C Natural Minor: C(60), D(62), Eb(63), F(65), G(67), Ab(68), Bb(70)
    assert!(minor.is_in_scale(60, 63)); // Eb
    assert!(minor.is_in_scale(60, 68)); // Ab
    assert!(!minor.is_in_scale(60, 64)); // E natural

    // D Dorian: D(62), E(64), F(65), G(67), A(69), B(71), C(72)
    assert!(dorian.is_in_scale(62, 71)); // B natural is in D dorian
    assert!(dorian.is_in_scale(62, 65)); // F natural

    // C Pentatonic Major: C, D, E, G, A
    assert!(pentatonic.is_in_scale(60, 60));
    assert!(pentatonic.is_in_scale(60, 62));
    assert!(pentatonic.is_in_scale(60, 64));
    assert!(pentatonic.is_in_scale(60, 67));
    assert!(pentatonic.is_in_scale(60, 69));
    assert!(!pentatonic.is_in_scale(60, 65)); // F is not in pentatonic major
}

#[test]
fn test_arpeggiator_directions() {
    let chord = vec![
        FlPianoRollNote::new(1, 0, 60, 0, 960), // C4
        FlPianoRollNote::new(2, 0, 64, 0, 960), // E4
        FlPianoRollNote::new(3, 0, 67, 0, 960), // G4
    ];

    // Arp Up
    let up = FlPianoRollTools::arpeggiate(&chord, ArpDirection::Up, 240, 960);
    assert_eq!(up.len(), 4);
    assert_eq!(up[0].key, 60);
    assert_eq!(up[1].key, 64);
    assert_eq!(up[2].key, 67);
    assert_eq!(up[3].key, 60);

    // Arp Down
    let down = FlPianoRollTools::arpeggiate(&chord, ArpDirection::Down, 240, 960);
    assert_eq!(down.len(), 4);
    assert_eq!(down[0].key, 67);
    assert_eq!(down[1].key, 64);
    assert_eq!(down[2].key, 60);
    assert_eq!(down[3].key, 67);

    // Arp UpDown: 60, 64, 67, 64
    let updown = FlPianoRollTools::arpeggiate(&chord, ArpDirection::UpDown, 240, 960);
    assert_eq!(updown.len(), 4);
    assert_eq!(updown[0].key, 60);
    assert_eq!(updown[1].key, 64);
    assert_eq!(updown[2].key, 67);
    assert_eq!(updown[3].key, 64);
}

#[test]
fn test_audio_scheduler_slide_note_processing() {
    use Crystal_wave::mixing::mixing_engine::MixingEngine;

    let mut scheduler = TimelineAudioScheduler::new(44100.0, 120.0);
    let track = Track::new(0, "Synth Track".to_string(), TrackType::Software);
    let tracks = vec![track];

    let mut mixer = MixingEngine::new(44100.0).expect("MixingEngine init failed");

    let base_note = FlPianoRollNote::new(1, 0, 60, 0, 960); // C4
    let slide_note = FlPianoRollNote::new(2, 0, 72, 240, 480).with_slide(true); // C5 slide
    let notes = vec![base_note, slide_note];

    let mut out_l = [0.0f32; 512];
    let mut out_r = [0.0f32; 512];

    // Process first block (starting at 0 ticks): base note generates audio
    scheduler.process_block(
        &tracks,
        &notes,
        &mut mixer,
        0,
        512,
        &mut out_l,
        &mut out_r,
    );

    let energy: f32 = out_l.iter().map(|s| s.abs()).sum();
    assert!(energy > 0.0, "Synthesizer output must not be silent");
}
