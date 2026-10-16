use Crystal_wave::ui::CrystalWaveApp;
use Crystal_wave::ui::canvas_timeline::SnapGrid;
use Crystal_wave::ui::message::UiMessage;
use Crystal_wave::ui::session_driver::UiSessionDriver;
use Crystal_wave::ui::theme::{ThemePreset, UiTheme};

#[test]
fn test_gui_ux_user_emulation_journey() {
    let mut driver = UiSessionDriver::new(48000.0);

    // 1. Initial State Check (Graphite Theme, Sixteenth snap, Stop state)
    assert_eq!(driver.app.theme.preset, ThemePreset::Graphite);
    assert_eq!(driver.app.snap_grid, SnapGrid::Sixteenth);
    assert!(!driver.app.is_playing);

    // 2. User presses Spacebar (Play), then toggles Loop (TransportToggleLoop)
    driver.dispatch_shortcut(' ');
    assert!(driver.app.is_playing);

    driver.dispatch(UiMessage::TransportToggleLoop);
    assert!(driver.app.arrangement.timeline.loop_region.enabled);

    // 3. User opens Command Palette with ⌘K / Ctrl+K
    driver.dispatch(UiMessage::ToggleCommandPalette);
    assert!(driver.app.command_palette_open);
    assert!(driver.app.command_query.is_empty());

    // 4. User types "orange" into search box
    driver.dispatch(UiMessage::CommandPaletteInput("orange".to_string()));
    assert_eq!(driver.app.command_query, "orange");

    // 5. User selects the filtered theme command
    driver.dispatch(UiMessage::ExecuteCommand("orange".to_string()));
    assert_eq!(driver.app.theme.preset, ThemePreset::FlDarkOrange);
    assert!(!driver.app.command_palette_open);

    // 6. User re-opens Palette and switches to Snap 1/16
    driver.dispatch(UiMessage::ToggleCommandPalette);
    driver.dispatch(UiMessage::ExecuteCommand("snap".to_string()));
    assert_eq!(driver.app.snap_grid, SnapGrid::Sixteenth);

    // 7. User navigates drawers (Piano Roll, Step Sequencer, Mixer)
    driver.dispatch(UiMessage::SelectDrawerTab(
        Crystal_wave::ui::layout::DrawerTab::StepSequencer,
    ));
    assert_eq!(
        driver.app.layout.active_drawer_tab,
        Crystal_wave::ui::layout::DrawerTab::StepSequencer
    );

    driver.dispatch(UiMessage::StepSequencerToggleStep {
        channel_id: 0,
        step_index: 2,
    });
    assert!(driver.app.channel_rack.channels[0].steps[2].active);

    // 8. User triggers full declarative UI element tree render (checking for zero panics)
    let _tree = driver.app.view();
}

#[test]
fn test_gui_ux_telemetry_and_status_bar() {
    let mut app = CrystalWaveApp::new(48000.0);
    app.playhead_tick = 3840 * 2 + 960; // Bar 3, Beat 2

    // Check status bar and tree render without panic
    let _element = app.view();
    assert_eq!(app.mixer.sample_rate, 48000.0);
}

#[test]
fn test_gui_ux_track_and_clip_lifecycle_journey() {
    let mut driver = UiSessionDriver::new(48000.0);
    let initial_tracks = driver.app.arrangement.timeline.tracks.len();

    // 1. User adds an Audio track and a MIDI track
    driver.dispatch(UiMessage::MenuAction(
        Crystal_wave::ui::message::MenuCommand::AddAudioTrack,
    ));
    driver.dispatch(UiMessage::MenuAction(
        Crystal_wave::ui::message::MenuCommand::AddMidiTrack,
    ));
    assert_eq!(
        driver.app.arrangement.timeline.tracks.len(),
        initial_tracks + 2
    );

    // 2. User selects Track 1, adjusts Fader dB and Pan
    driver.dispatch(UiMessage::SelectTrack(1));
    assert_eq!(driver.app.arrangement.selected_track_index, Some(1));

    driver.dispatch(UiMessage::SetTrackFader {
        track_id: 1,
        volume_db: -3.5,
    });
    driver.dispatch(UiMessage::SetTrackPan {
        track_id: 1,
        pan: -0.25,
    });
    assert!((driver.app.mixer.channels[1].volume_fader_db - (-3.5)).abs() < 0.001);
    assert!((driver.app.mixer.channels[1].pan.balance - (-0.25)).abs() < 0.001);

    // 3. User toggles Mute, Solo, and Record Arm
    driver.dispatch(UiMessage::ToggleTrackMute(1));
    assert!(driver.app.mixer.channels[1].muted);

    driver.dispatch(UiMessage::ToggleTrackSolo(1));
    assert!(driver.app.mixer.channels[1].solo);

    driver.dispatch(UiMessage::ToggleTrackRecordArm(1));
    assert!(driver.app.arrangement.timeline.tracks[1].record_armed);

    // 4. User adds a pattern clip on Track 1 and adjusts fades
    let initial_regions = driver.app.arrangement.timeline.tracks[1].regions.len();
    driver.dispatch(UiMessage::TimelineAddPatternClip {
        track_id: 1,
        start_tick: 7680,
        length_ticks: 3840,
        name: "Verse Drums".to_string(),
    });
    assert_eq!(
        driver.app.arrangement.timeline.tracks[1].regions.len(),
        initial_regions + 1
    );

    let clip_id = driver.app.arrangement.timeline.tracks[1]
        .regions
        .last()
        .expect("must have added region")
        .id;
    driver.dispatch(UiMessage::TimelineFadeClip {
        clip_id,
        fade_in_ticks: Some(240),
        fade_out_ticks: Some(480),
    });
    let last_region = driver.app.arrangement.timeline.tracks[1]
        .regions
        .last()
        .expect("must have region");
    assert_eq!(last_region.fade.in_ticks, 240);
    assert_eq!(last_region.fade.out_ticks, 480);

    // 5. User duplicates clip
    driver.dispatch(UiMessage::TimelineDuplicateClip(clip_id));
    assert_eq!(
        driver.app.arrangement.timeline.tracks[1].regions.len(),
        initial_regions + 2
    );

    // 6. User verifies view renders cleanly with duplicated clips
    let _tree = driver.app.view();
}

#[test]
fn test_gui_ux_piano_roll_and_drawer_render_integrity() {
    let mut driver = UiSessionDriver::new(48000.0);

    // 1. Switch to Piano Roll drawer and add notes
    driver.dispatch(UiMessage::SelectDrawerTab(
        Crystal_wave::ui::layout::DrawerTab::PianoRoll,
    ));
    assert_eq!(
        driver.app.layout.active_drawer_tab,
        Crystal_wave::ui::layout::DrawerTab::PianoRoll
    );

    driver.dispatch(UiMessage::PianoRollAddNote {
        channel_id: 0,
        key: 60, // C4
        start_tick: 0,
        length_ticks: 480,
    });
    driver.dispatch(UiMessage::PianoRollAddNote {
        channel_id: 0,
        key: 64, // E4
        start_tick: 480,
        length_ticks: 480,
    });

    // 2. Perform FL-style articulations
    driver.dispatch(UiMessage::PianoRollQuickStrum);
    driver.dispatch(UiMessage::PianoRollQuickChop);
    driver.dispatch(UiMessage::PianoRollFlipHorizontal);
    driver.dispatch(UiMessage::PianoRollFlipVertical);

    // 3. Cycle through all drawer views and verify view tree rendering
    let drawer_tabs = [
        Crystal_wave::ui::layout::DrawerTab::PianoRoll,
        Crystal_wave::ui::layout::DrawerTab::StepSequencer,
        Crystal_wave::ui::layout::DrawerTab::Automation,
        Crystal_wave::ui::layout::DrawerTab::Mixer,
    ];

    for tab in drawer_tabs {
        driver.dispatch(UiMessage::SelectDrawerTab(tab));
        let _rendered_tree = driver.app.view();
    }
}

#[test]
fn test_gui_ux_anti_slop_and_contrast_tokens() {
    let graphite = UiTheme::graphite();
    let fl_orange = UiTheme::fl_dark_orange();

    // Verify contrast between background and text_primary
    assert!(graphite.text_primary.r > 0.8 && graphite.text_primary.g > 0.8);
    assert!(graphite.background.r < 0.2 && graphite.background.g < 0.2);

    assert!(fl_orange.text_primary.r > 0.8 && fl_orange.text_primary.g > 0.8);
    assert!(fl_orange.background.r < 0.2 && fl_orange.background.g < 0.2);

    // Verify VFD OLED display token integrity
    let oled_graphite = graphite.oled_vfd_display_style();
    let oled_orange = fl_orange.oled_vfd_display_style();
    assert!(oled_graphite.background.is_some());
    assert!(oled_orange.background.is_some());
}
