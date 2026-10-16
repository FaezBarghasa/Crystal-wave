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
