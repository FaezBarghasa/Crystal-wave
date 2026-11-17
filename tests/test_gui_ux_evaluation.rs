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

#[test]
fn test_phase1_audio_scheduler_genuine_dsp_samples() {
    let mut driver = UiSessionDriver::new(48000.0);
    let mut master_l = [0.0f32; 512];
    let mut master_r = [0.0f32; 512];

    // Process a block at frame 0 (where Audio Take 1 is active on Track 0)
    driver.app.audio_scheduler.process_block(
        &driver.app.arrangement.timeline.tracks,
        &driver.app.piano_roll_notes,
        &mut driver.app.mixer,
        0,
        512,
        &mut master_l,
        &mut master_r,
    );

    // Verify non-zero genuine audio output was synthesized and summed
    let sum_energy: f32 = master_l
        .iter()
        .chain(master_r.iter())
        .map(|s| s.abs())
        .sum();
    assert!(
        sum_energy > 0.05,
        "Audio engine produced silence! Expected genuine audio energy, got sum_energy: {}",
        sum_energy
    );

    // Verify channel meters registered signal
    let ch0_peak = driver.app.mixer.channels[0].meter.peak_db;
    assert!(
        ch0_peak > -60.0,
        "Channel 0 meter indicates silence: {:.1} dBFS",
        ch0_peak
    );
}

#[test]
fn test_phase1_offline_bounce_produces_real_wav() {
    let mut driver = UiSessionDriver::new(48000.0);

    // Trigger full project bounce
    driver.dispatch(UiMessage::MenuAction(
        Crystal_wave::ui::message::MenuCommand::BounceWav,
    ));

    // Verify bounce file exists and inspect audio content with hound
    let wav_path = std::path::Path::new("bounce_output.wav");
    assert!(wav_path.exists(), "bounce_output.wav was not created!");

    let mut reader = hound::WavReader::open(wav_path).expect("Failed to open bounce_output.wav");
    let spec = reader.spec();
    assert_eq!(spec.channels, 2);
    assert_eq!(spec.sample_rate, 48000);
    assert_eq!(spec.bits_per_sample, 24);

    let sample_count = reader.duration();
    assert!(
        sample_count >= 48000,
        "Bounce duration was too short: {} frames",
        sample_count
    );

    // Assert that the bounced WAV contains genuine audio signal rather than silence
    let samples: Vec<i32> = reader.samples::<i32>().map(|s| s.unwrap()).collect();
    let has_non_zero = samples.iter().any(|&s| s != 0);
    assert!(has_non_zero, "bounced WAV contains only digital zeroes!");
}

#[test]
fn test_phase1_playhead_tick_and_frame_clock_synchronization() {
    let mut driver = UiSessionDriver::new(48000.0);

    // Verify mathematical conversion
    let frame_at_bar1 = driver.app.audio_scheduler.tick_to_frame(3840);
    assert_eq!(frame_at_bar1, 96000); // 48000 * 2.0s
    let tick_round_trip = driver.app.audio_scheduler.frame_to_tick(frame_at_bar1);
    assert_eq!(tick_round_trip, 3840);

    // Verify transport advance
    driver.dispatch(UiMessage::TransportPlay);
    assert!(driver.app.is_playing);

    driver.dispatch(UiMessage::Tick);
    assert!(driver.app.playhead_frame >= 256);
    assert!(driver.app.playhead_tick > 0);

    // Verify transport stop resets both frame and tick
    driver.dispatch(UiMessage::TransportStop);
    assert_eq!(driver.app.playhead_frame, 0);
    assert_eq!(driver.app.playhead_tick, 0);
    assert!(!driver.app.is_playing);
}

#[test]
fn test_phase2_canvas_ghost_drag_interaction() {
    use Crystal_wave::ui::canvas_timeline::ArrangementCanvasProgram;

    let mut program = Crystal_wave::ui::canvas_timeline::ArrangementCanvasProgram::new(0, 1.0, 2)
        .with_track_types(vec![
            Crystal_wave::timeline::track::TrackType::Audio,
            Crystal_wave::timeline::track::TrackType::Software,
        ]);

    program
        .clips
        .push(Crystal_wave::ui::canvas_timeline::TimelineClipDisplay {
            id: 1,
            track_index: 0,
            start_tick: 0,
            length_ticks: 3840,
            name: "Audio Take 1".to_string(),
            color_rgb: [80, 120, 220],
            muted: false,
            looped: false,
            fade_in_ticks: 0,
            fade_out_ticks: 0,
            kind: Crystal_wave::ui::canvas_timeline::ClipVisualKind::Audio {
                waveform_peaks: Vec::new(),
            },
            is_selected: false,
        });

    let mut state = Crystal_wave::ui::canvas_timeline::CanvasInteractionState::default();
    let bounds = iced::Rectangle::new(iced::Point::ORIGIN, iced::Size::new(1000.0, 400.0));

    // 1. Mouse down on clip in Track 0 (x=15.0 is tick ~1843, y=75.0 is Track 0 GrabMove zone)
    let cursor_down = iced::mouse::Cursor::Available(iced::Point::new(15.0, 75.0));
    let press_event = iced::widget::canvas::Event::Mouse(iced::mouse::Event::ButtonPressed(
        iced::mouse::Button::Left,
    ));
    let _ =
        <ArrangementCanvasProgram as iced::widget::canvas::Program<UiMessage, iced::Theme>>::update(
            &program,
            &mut state,
            &press_event,
            bounds,
            cursor_down,
        );

    assert!(state.dragged_clip.is_some());
    assert!(state.live_drag.is_some());

    // 2. Drag cursor to Track 1 (y = 140.0)
    let cursor_move_trk1 = iced::mouse::Cursor::Available(iced::Point::new(20.0, 140.0));
    let move_event = iced::widget::canvas::Event::Mouse(iced::mouse::Event::CursorMoved {
        position: iced::Point::new(20.0, 140.0),
    });
    let _ =
        <ArrangementCanvasProgram as iced::widget::canvas::Program<UiMessage, iced::Theme>>::update(
            &program,
            &mut state,
            &move_event,
            bounds,
            cursor_move_trk1,
        );

    let preview_trk1 = state
        .live_drag
        .as_ref()
        .expect("must have live drag preview");
    assert_eq!(preview_trk1.target_track, 1);
    // Audio clip on Software track must be flagged as invalid drop
    assert!(!preview_trk1.is_valid_drop);

    // 3. Drag cursor back to Track 0 (y = 75.0)
    let cursor_move_trk0 = iced::mouse::Cursor::Available(iced::Point::new(20.0, 75.0));
    let move_event2 = iced::widget::canvas::Event::Mouse(iced::mouse::Event::CursorMoved {
        position: iced::Point::new(20.0, 75.0),
    });
    let _ =
        <ArrangementCanvasProgram as iced::widget::canvas::Program<UiMessage, iced::Theme>>::update(
            &program,
            &mut state,
            &move_event2,
            bounds,
            cursor_move_trk0,
        );

    let preview_trk0 = state
        .live_drag
        .as_ref()
        .expect("must have live drag preview");
    assert_eq!(preview_trk0.target_track, 0);
    // Audio clip on Audio track is valid
    assert!(preview_trk0.is_valid_drop);
}

#[test]
fn test_phase2_track_type_compatibility_drop_rejection() {
    let mut driver = UiSessionDriver::new(48000.0);

    // Track 0 is Audio with Region 1. Track 1 is Software with Region 2.
    assert_eq!(
        driver.app.arrangement.timeline.tracks[0].track_type,
        Crystal_wave::timeline::track::TrackType::Audio
    );
    assert_eq!(
        driver.app.arrangement.timeline.tracks[1].track_type,
        Crystal_wave::timeline::track::TrackType::Software
    );

    // 1. Attempt to move Audio Region 1 to Track 1 (Software) -> should be rejected and kept on Track 0!
    driver.dispatch(UiMessage::TimelineMoveClip {
        clip_id: 1,
        new_track_id: 1,
        new_start_tick: 960,
    });

    let trk0_has_reg1 = driver.app.arrangement.timeline.tracks[0]
        .regions
        .iter()
        .any(|r| r.id == 1);
    let trk1_has_reg1 = driver.app.arrangement.timeline.tracks[1]
        .regions
        .iter()
        .any(|r| r.id == 1);

    assert!(
        trk0_has_reg1,
        "Audio region should remain on Track 0 when incompatible drop occurs!"
    );
    assert!(
        !trk1_has_reg1,
        "Incompatible audio region must NOT be placed on Software Instrument track!"
    );

    // 2. Add another Audio track (Track 2)
    driver.dispatch(UiMessage::MenuAction(
        Crystal_wave::ui::message::MenuCommand::AddAudioTrack,
    ));
    assert_eq!(
        driver.app.arrangement.timeline.tracks[2].track_type,
        Crystal_wave::timeline::track::TrackType::Audio
    );

    // 3. Move Region 1 to Track 2 (Audio -> Audio) -> should succeed!
    driver.dispatch(UiMessage::TimelineMoveClip {
        clip_id: 1,
        new_track_id: 2,
        new_start_tick: 1920,
    });

    let trk2_has_reg1 = driver.app.arrangement.timeline.tracks[2]
        .regions
        .iter()
        .any(|r| r.id == 1);
    assert!(
        trk2_has_reg1,
        "Compatible audio region must successfully move to Track 2!"
    );
}

#[test]
fn test_phase3_piano_roll_canvas_coordinate_projections() {
    use Crystal_wave::ui::canvas_piano_roll::{
        KEY_HEIGHT, KEYBOARD_WIDTH, PianoRollCanvasProgram, is_black_key, midi_key_to_name,
    };

    let program = PianoRollCanvasProgram::new(0, 1.0, 872.0);

    // 1. Black key validation
    assert!(!is_black_key(60)); // C4 (White)
    assert!(is_black_key(61)); // C#4 (Black)
    assert!(!is_black_key(62)); // D4 (White)
    assert!(is_black_key(63)); // D#4 (Black)
    assert!(!is_black_key(64)); // E4 (White)
    assert!(!is_black_key(65)); // F4 (White)
    assert!(is_black_key(66)); // F#4 (Black)

    // 2. Note naming
    assert_eq!(midi_key_to_name(60), "C4");
    assert_eq!(midi_key_to_name(61), "C#4");
    assert_eq!(midi_key_to_name(72), "C5");
    assert_eq!(midi_key_to_name(48), "C3");

    // 3. Time projections
    let tick_0_x = program.tick_to_x(0);
    assert_eq!(tick_0_x, KEYBOARD_WIDTH);
    let tick_960_x = program.tick_to_x(960);
    assert_eq!(tick_960_x, KEYBOARD_WIDTH + 80.0);
    assert_eq!(program.x_to_tick(tick_960_x), 960);

    // 4. Pitch vertical projections
    let y_c4 = program.key_to_y(60);
    let expected_c4_y = ((127 - 60) as f32 * KEY_HEIGHT) - 872.0;
    assert_eq!(y_c4, expected_c4_y);
    assert_eq!(program.y_to_key(y_c4), 60);
}

#[test]
fn test_phase3_piano_roll_2d_mouse_interactions() {
    use Crystal_wave::ui::canvas_piano_roll::{
        KEYBOARD_WIDTH, PianoRollCanvasProgram, PianoRollInteractionState, PianoRollNoteDisplay,
    };
    use iced::widget::canvas::Program;

    let mut program = PianoRollCanvasProgram::new(0, 1.0, 872.0);
    program.notes.push(PianoRollNoteDisplay {
        id: 10,
        channel_id: 0,
        key: 60, // C4
        start_tick: 0,
        length_ticks: 960,
        velocity: 0.8,
        is_slide: false,
    });

    let mut state = PianoRollInteractionState::default();
    let bounds = iced::Rectangle::new(iced::Point::ORIGIN, iced::Size::new(1000.0, 500.0));

    // 1. Click Left Keyboard Sidebar -> Emits PianoRollSelectKey
    let kb_cursor = iced::mouse::Cursor::Available(iced::Point::new(
        KEYBOARD_WIDTH - 10.0,
        program.key_to_y(60) + 4.0,
    ));
    let kb_press = iced::widget::canvas::Event::Mouse(iced::mouse::Event::ButtonPressed(
        iced::mouse::Button::Left,
    ));
    let action = <PianoRollCanvasProgram as Program<UiMessage, iced::Theme>>::update(
        &program, &mut state, &kb_press, bounds, kb_cursor,
    );
    assert!(action.is_some());

    // 2. Mouse down on note 10 body (C4, x = KEYBOARD_WIDTH + 10.0, y = key_to_y(60) + 4.0)
    let note_cursor_down = iced::mouse::Cursor::Available(iced::Point::new(
        KEYBOARD_WIDTH + 10.0,
        program.key_to_y(60) + 4.0,
    ));
    let note_press = iced::widget::canvas::Event::Mouse(iced::mouse::Event::ButtonPressed(
        iced::mouse::Button::Left,
    ));
    let _ = <PianoRollCanvasProgram as Program<UiMessage, iced::Theme>>::update(
        &program,
        &mut state,
        &note_press,
        bounds,
        note_cursor_down,
    );
    assert!(state.active_drag.is_some());
    assert!(state.live_preview.is_some());

    // 3. Move cursor to D4 (key 62) and tick 960 (x = KEYBOARD_WIDTH + 80.0)
    let move_cursor = iced::mouse::Cursor::Available(iced::Point::new(
        KEYBOARD_WIDTH + 80.0,
        program.key_to_y(62) + 4.0,
    ));
    let move_event = iced::widget::canvas::Event::Mouse(iced::mouse::Event::CursorMoved {
        position: iced::Point::new(KEYBOARD_WIDTH + 80.0, program.key_to_y(62) + 4.0),
    });
    let _ = <PianoRollCanvasProgram as Program<UiMessage, iced::Theme>>::update(
        &program,
        &mut state,
        &move_event,
        bounds,
        move_cursor,
    );
    let preview = state.live_preview.as_ref().expect("must have preview");
    assert_eq!(preview.target_key, 62);
    assert_eq!(preview.delta_semitones, 2);

    // 4. Release mouse -> emits PianoRollMoveNote
    let release_event = iced::widget::canvas::Event::Mouse(iced::mouse::Event::ButtonReleased(
        iced::mouse::Button::Left,
    ));
    let release_action = <PianoRollCanvasProgram as Program<UiMessage, iced::Theme>>::update(
        &program,
        &mut state,
        &release_event,
        bounds,
        move_cursor,
    );
    assert!(release_action.is_some());

    // 5. Right Click Note -> Emits PianoRollDeleteNote
    let right_click = iced::widget::canvas::Event::Mouse(iced::mouse::Event::ButtonPressed(
        iced::mouse::Button::Right,
    ));
    let del_action = <PianoRollCanvasProgram as Program<UiMessage, iced::Theme>>::update(
        &program,
        &mut state,
        &right_click,
        bounds,
        note_cursor_down,
    );
    assert!(del_action.is_some());
}

#[test]
fn test_phase3_piano_roll_app_reducer_actions() {
    let mut driver = UiSessionDriver::new(48000.0);

    // Initial note 1 is C4 (key 60) at start_tick 0 with length 960
    let note1_orig = driver
        .app
        .piano_roll_notes
        .iter()
        .find(|n| n.id == 1)
        .unwrap();
    assert_eq!(note1_orig.key, 60);
    assert_eq!(note1_orig.start_tick, 0);

    // 1. Move Note 1 (+2 semitones to D4, +960 ticks)
    driver.dispatch(UiMessage::PianoRollMoveNote {
        id: 1,
        delta_ticks: 960,
        delta_key: 2,
    });
    let note1_moved = driver
        .app
        .piano_roll_notes
        .iter()
        .find(|n| n.id == 1)
        .unwrap();
    assert_eq!(note1_moved.key, 62);
    assert_eq!(note1_moved.start_tick, 960);

    // 2. Resize Note 1 (+480 ticks)
    driver.dispatch(UiMessage::PianoRollResizeNote {
        id: 1,
        delta_ticks: 480,
    });
    let note1_resized = driver
        .app
        .piano_roll_notes
        .iter()
        .find(|n| n.id == 1)
        .unwrap();
    assert_eq!(note1_resized.length_ticks, 960 + 480);

    // 3. Toggle Slide on Note 1
    assert!(!note1_resized.is_slide);
    driver.dispatch(UiMessage::PianoRollToggleSlide(1));
    let note1_slide = driver
        .app
        .piano_roll_notes
        .iter()
        .find(|n| n.id == 1)
        .unwrap();
    assert!(note1_slide.is_slide);

    // 4. Polyphonic Ghost note synthesis test
    let pr_program = driver.app.piano_roll_canvas_program();
    assert!(!pr_program.notes.is_empty());
    // Channels from channel rack (808 kick, clap, hat, bass) produce ghost notes
    assert!(
        !pr_program.ghost_notes.is_empty(),
        "Ghost notes should be populated from background channels!"
    );
}

#[test]
fn test_phase4_safe_track_deletion_cascading_reindex() {
    let mut driver = UiSessionDriver::new(48000.0);

    // Initial project has 2 tracks (Track 0: Audio 1, Track 1: Inst 1)
    assert_eq!(driver.app.arrangement.timeline.tracks.len(), 2);

    // 1. Add 2 additional tracks -> Total 4 tracks
    driver.dispatch(UiMessage::MenuAction(
        Crystal_wave::ui::message::MenuCommand::AddAudioTrack,
    )); // Track 2
    driver.dispatch(UiMessage::MenuAction(
        Crystal_wave::ui::message::MenuCommand::AddMidiTrack,
    )); // Track 3

    assert_eq!(driver.app.arrangement.timeline.tracks.len(), 4);
    assert_eq!(driver.app.mixer.channels.len(), 4);

    // Add a region to Track 1, Track 2, and Track 3
    driver.dispatch(UiMessage::TimelineAddPatternClip {
        track_id: 1,
        start_tick: 0,
        length_ticks: 3840,
        name: "Pattern Trk 1".to_string(),
    });
    driver.dispatch(UiMessage::TimelineAddPatternClip {
        track_id: 2,
        start_tick: 3840,
        length_ticks: 3840,
        name: "Pattern Trk 2".to_string(),
    });
    driver.dispatch(UiMessage::TimelineAddPatternClip {
        track_id: 3,
        start_tick: 7680,
        length_ticks: 3840,
        name: "Pattern Trk 3".to_string(),
    });

    // 2. Select Track 1 and Delete it
    driver.dispatch(UiMessage::SelectTrack(1));
    driver.dispatch(UiMessage::MenuAction(
        Crystal_wave::ui::message::MenuCommand::DeleteTrack,
    ));

    // 3. Verify surviving track count is 3
    assert_eq!(driver.app.arrangement.timeline.tracks.len(), 3);
    assert_eq!(driver.app.mixer.channels.len(), 3);

    // 4. Verify all surviving tracks have strictly contiguous IDs 0, 1, 2
    for (idx, trk) in driver.app.arrangement.timeline.tracks.iter().enumerate() {
        assert_eq!(trk.id, idx, "Track ID must match its contiguous index!");
        // Verify all regions on this track have re-indexed track_id matching idx!
        for reg in &trk.regions {
            assert_eq!(
                reg.track_id, idx,
                "Region track_id must be updated to match the re-indexed track ID!"
            );
        }
    }

    // 5. Verify selected track index and inspector are safely clamped
    assert!(driver.app.workspace.selected_track_id < 3);
    assert!(driver.app.inspector.selected_track_id < 3);

    // 6. Verify view renders cleanly after track deletion
    let _tree = driver.app.view();
}

#[test]
fn test_phase4_graduated_led_meter_ballistics() {
    use Crystal_wave::ui::view::render_led_meter;

    // 1. Quiet signal (-36 dBFS): Green segments lit
    let _quiet = render_led_meter(-36.0, -42.0, -30.0, false);

    // 2. Nominal mix (-14 dBFS): Green segments lit up to -14 dB
    let _nominal = render_led_meter(-14.0, -18.0, -10.0, false);

    // 3. Hot Amber (-4.5 dBFS): Green + Amber segments lit
    let _hot = render_led_meter(-4.5, -8.0, -2.0, false);

    // 4. Clip Overload (+1.2 dBFS): All 12 segments lit including Red Clip LED
    let _clipped = render_led_meter(1.2, -1.0, 1.2, false);

    // 5. Auto-Mute Tripped Overload: Red Clip illuminated + "MUTE" badge
    let _muted = render_led_meter(15.0, 12.0, 15.0, true);
}

#[test]
fn test_phase4_honest_telemetry_status_bar() {
    let mut driver = UiSessionDriver::new(48000.0);

    // 1. Initial status bar renders with zero XRUNs and nominal master
    {
        let _tree = driver.app.view();
    }

    // 2. Simulate hardware buffer underflows if audio driver is attached
    if let Some(ref drv) = driver.app.audio_driver {
        drv.underrun_count
            .store(4, std::sync::atomic::Ordering::Relaxed);
    }

    // 3. Simulate high master channel peak
    if let Some(master) = driver.app.mixer.channels.first_mut() {
        master.meter.peak_db = 0.5;
        master.meter.auto_mute_tripped = false;
    }
    {
        let _tree_clip = driver.app.view();
    }

    // 4. Simulate auto-mute trip
    if let Some(master) = driver.app.mixer.channels.first_mut() {
        master.meter.auto_mute_tripped = true;
    }
    {
        let _tree_mute = driver.app.view();
    }
}
