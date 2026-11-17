use Crystal_wave::ui::CrystalWaveApp;
use Crystal_wave::ui::canvas_timeline::{ArrangementCanvasProgram, PPQN, SnapGrid, TICKS_PER_BAR};
use Crystal_wave::ui::layout::{DockPosition, DrawerTab, LayoutState, PanelId};
use Crystal_wave::ui::message::UiMessage;
use Crystal_wave::ui::theme::{ThemePreset, UiTheme};

#[test]
fn test_ui_theme_tokens_and_presets() {
    let graphite = UiTheme::graphite();
    assert_eq!(graphite.preset, ThemePreset::Graphite);
    assert_eq!(graphite.note_colors.len(), 16);
    assert_eq!(graphite.meters.len(), 6);

    let fl_orange = UiTheme::fl_dark_orange();
    assert_eq!(fl_orange.preset, ThemePreset::FlDarkOrange);
    assert_eq!(fl_orange.note_colors.len(), 16);
    assert_eq!(fl_orange.meters.len(), 6);

    let fl_modern = UiTheme::fl_studio_modern();
    assert_eq!(fl_modern.preset, ThemePreset::FlStudioModern);
    assert_eq!(fl_modern.note_colors.len(), 16);
    assert_eq!(fl_modern.meters.len(), 6);
    assert_eq!(fl_modern.accent, iced::Color::from_rgb8(0xff, 0x88, 0x00));
    assert_eq!(
        fl_modern.background,
        iced::Color::from_rgb8(0x10, 0x11, 0x14)
    );
    assert_eq!(fl_modern.surface, iced::Color::from_rgb8(0x18, 0x1a, 0x20));
    assert_eq!(fl_modern.step_odd, iced::Color::from_rgb8(0x18, 0x1a, 0x20));
    assert_eq!(
        fl_modern.step_even,
        iced::Color::from_rgb8(0x25, 0x29, 0x32)
    );

    // FL Orange accent verification
    assert_eq!(
        fl_orange.border_focus,
        iced::Color::from_rgb8(0xf8, 0x8a, 0x36)
    );
    assert_eq!(fl_orange.accent, iced::Color::from_rgb8(0xf8, 0x8a, 0x36));

    // Note colors rotation
    assert_eq!(fl_orange.note_color(0), fl_orange.note_colors[0]);
    assert_eq!(fl_orange.note_color(16), fl_orange.note_colors[0]);
    assert_eq!(fl_modern.note_color(0), fl_modern.note_colors[0]);

    // Container style builders
    let panel_style = fl_orange.panel_container_style();
    assert!(panel_style.background.is_some());
    let header_style = fl_orange.header_container_style();
    assert!(header_style.background.is_some());

    // Double-bezel & Machined Capsules
    let bezel_style = fl_modern.outer_bezel_style();
    assert!(bezel_style.background.is_some());
    let well_style = fl_modern.inner_well_style();
    assert!(well_style.background.is_some());
    let capsule_style = fl_modern.machined_capsule_style();
    assert!(capsule_style.background.is_some());

    // Luxury Hardware & OLED Display Containers
    let oled_style = fl_orange.oled_vfd_display_style();
    assert!(oled_style.background.is_some());
    let oled_modern = fl_modern.oled_vfd_display_style();
    assert!(oled_modern.background.is_some());
    let rack_style = graphite.hardware_rack_style();
    assert!(rack_style.background.is_some());

    // Studio Meter Color Mapping (-60 dB to +6 dB clip)
    assert_eq!(graphite.meter_color_for_db(-30.0), graphite.meters[0]);
    assert_eq!(graphite.meter_color_for_db(-15.0), graphite.meters[1]);
    assert_eq!(graphite.meter_color_for_db(-8.0), graphite.meters[2]);
    assert_eq!(graphite.meter_color_for_db(-4.0), graphite.meters[3]);
    assert_eq!(graphite.meter_color_for_db(-1.5), graphite.meters[4]);
    assert_eq!(graphite.meter_color_for_db(0.5), graphite.meters[5]); // Clip Red

    // Curated Track Palette
    let track_0 = graphite.track_accent_color(0);
    let track_1 = graphite.track_accent_color(1);
    assert_ne!(track_0, track_1);
}

#[test]
fn test_layout_state_docking_and_detachment() {
    let mut layout = LayoutState::default();

    // Default states
    assert!(layout.browser.visible);
    assert_eq!(layout.browser.dock, DockPosition::Left);
    assert_eq!(layout.active_drawer_tab, DrawerTab::PianoRoll);
    assert!(!layout.is_detached(PanelId::Mixer));

    // Tab switching
    layout.set_drawer_tab(DrawerTab::StepSequencer);
    assert_eq!(layout.active_drawer_tab, DrawerTab::StepSequencer);
    assert!(layout.bottom_drawer.visible);

    // Panel toggling
    layout.toggle_panel(PanelId::Inspector);
    assert!(layout.inspector.visible);
    layout.toggle_panel(PanelId::Inspector);
    assert!(!layout.inspector.visible);

    // Detach to OS Window
    layout.detach_to_window(PanelId::Mixer);
    assert!(layout.is_detached(PanelId::Mixer));
    assert_eq!(layout.mixer_panel.dock, DockPosition::Float);

    // Redock back to layout
    layout.redock_to_main(PanelId::Mixer, DockPosition::Bottom);
    assert!(!layout.is_detached(PanelId::Mixer));
    assert_eq!(layout.mixer_panel.dock, DockPosition::Bottom);
}

#[test]
fn test_snap_grid_calculations() {
    let snap_bar = SnapGrid::Bar;
    assert_eq!(snap_bar.snap(0, 1.0), 0);
    assert_eq!(snap_bar.snap(100, 1.0), 0);
    assert_eq!(snap_bar.snap(2000, 1.0), TICKS_PER_BAR);
    assert_eq!(snap_bar.snap(3840, 1.0), 3840);

    let snap_16th = SnapGrid::Sixteenth;
    assert_eq!(snap_16th.snap_step_ticks(1.0), 240);
    assert_eq!(snap_16th.snap(120, 1.0), 240);
    assert_eq!(snap_16th.snap(119, 1.0), 0);
    assert_eq!(snap_16th.snap(240, 1.0), 240);
    assert_eq!(snap_16th.snap(350, 1.0), 240);
    assert_eq!(snap_16th.snap(370, 1.0), 480);

    let snap_smart = SnapGrid::Smart;
    // High zoom gives fine 32nd notes
    assert_eq!(snap_smart.snap_step_ticks(5.0), PPQN / 8);
    // Low zoom gives 1 beat
    assert_eq!(snap_smart.snap_step_ticks(0.5), PPQN);

    let snap_off = SnapGrid::Off;
    assert_eq!(snap_off.snap(1234, 1.0), 1234);
}

#[test]
fn test_app_theme_and_snap_switching() {
    let mut app = CrystalWaveApp::new(44100.0);
    assert_eq!(app.theme.preset, ThemePreset::Graphite);
    assert_eq!(app.snap_grid, SnapGrid::Sixteenth);

    // Switch to FL Studio theme
    app.update(UiMessage::SetThemePreset(ThemePreset::FlDarkOrange));
    assert_eq!(app.theme.preset, ThemePreset::FlDarkOrange);

    // Switch Snap
    app.update(UiMessage::SetSnapGrid(SnapGrid::Bar));
    assert_eq!(app.snap_grid, SnapGrid::Bar);

    // Switch Drawer Tab
    app.update(UiMessage::SelectDrawerTab(DrawerTab::StepSequencer));
    assert_eq!(app.layout.active_drawer_tab, DrawerTab::StepSequencer);
}

#[test]
fn test_step_sequencer_and_piano_roll_interactions() {
    let mut app = CrystalWaveApp::new(44100.0);

    // Channel rack step toggling
    let kick_id = 0;
    let initial_state = app.channel_rack.channels[kick_id].steps[0].active;
    app.update(UiMessage::StepSequencerToggleStep {
        channel_id: kick_id,
        step_index: 0,
    });
    assert_ne!(
        app.channel_rack.channels[kick_id].steps[0].active,
        initial_state
    );

    // Piano Roll add note
    let prev_len = app.piano_roll_notes.len();
    app.update(UiMessage::PianoRollAddNote {
        channel_id: 0,
        key: 72, // C5
        start_tick: 3840,
        length_ticks: 960,
    });
    assert_eq!(app.piano_roll_notes.len(), prev_len + 1);

    // Piano Roll delete note
    let last_id = app.piano_roll_notes.last().unwrap().id;
    app.update(UiMessage::PianoRollDeleteNote(last_id));
    assert_eq!(app.piano_roll_notes.len(), prev_len);
}

#[test]
fn test_timeline_marquee_and_clip_duplication() {
    let mut app = CrystalWaveApp::new(44100.0);
    let trk0_clips_before = app.arrangement.timeline.tracks[0].regions.len();

    // Duplicate clip 1
    app.update(UiMessage::TimelineDuplicateClip(1));
    assert_eq!(
        app.arrangement.timeline.tracks[0].regions.len(),
        trk0_clips_before + 1
    );

    // Marquee select across ticks 0..4000
    app.update(UiMessage::TimelineMarqueeSelect {
        start_tick: 0,
        end_tick: 4000,
        start_track: 0,
        end_track: 1,
    });
    assert!(app.marquee_selection.is_some());
    let (t1, t2, trk1, trk2) = app.marquee_selection.unwrap();
    assert_eq!(t1, 0);
    assert_eq!(t2, 4000);
    assert_eq!(trk1, 0);
    assert_eq!(trk2, 1);
}

#[test]
fn test_command_palette_execution() {
    let mut app = CrystalWaveApp::new(44100.0);
    assert!(!app.command_palette_open);

    app.update(UiMessage::ToggleCommandPalette);
    assert!(app.command_palette_open);

    // Execute theme command via palette
    app.update(UiMessage::ExecuteCommand("orange".to_string()));
    assert!(!app.command_palette_open);
    assert_eq!(app.theme.preset, ThemePreset::FlDarkOrange);

    // Switch to modern FL 2026 theme via command palette
    app.update(UiMessage::ToggleCommandPalette);
    app.update(UiMessage::ExecuteCommand("modern".to_string()));
    assert!(!app.command_palette_open);
    assert_eq!(app.theme.preset, ThemePreset::FlStudioModern);

    // Execute play command via palette
    app.update(UiMessage::ExecuteCommand("play".to_string()));
    assert!(app.is_playing);
}

#[test]
fn test_canvas_timeline_program_configuration() {
    let program = ArrangementCanvasProgram::new(1920, 1.5, 4)
        .with_snap(SnapGrid::Eighth)
        .with_theme(true)
        .with_scroll_y(80.0)
        .with_marquee(Some((0, 3840, 0, 2)));

    assert_eq!(program.snap, SnapGrid::Eighth);
    assert!(program.is_fl_theme);
    assert_eq!(program.scroll_offset_y, 80.0);
    assert!(program.marquee_selection.is_some());

    // Coordinate conversion
    let tick = program.x_to_tick(100.0, 1000.0);
    assert!(tick > 0);
    let track = program.y_to_track(150.0);
    assert!(track.is_some());
}
