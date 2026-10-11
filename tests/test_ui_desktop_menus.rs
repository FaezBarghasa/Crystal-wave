use Crystal_wave::ui::app::CrystalWaveApp;
use Crystal_wave::ui::message::{MenuCommand, UiMessage};
use Crystal_wave::ui_state::EditorDrawerTab;

#[test]
fn test_ui_menu_actions_and_drawer_state() {
    let mut app = CrystalWaveApp::new(44100.0);
    assert_eq!(app.mixer.channels.len(), 2); // Audio 1, Inst 1
    assert!(!app.workspace.mixer_drawer_open);
    assert!(app.workspace.inspector_open); // Open by default

    // 1. Add Audio Track via MenuAction
    app.update(UiMessage::MenuAction(MenuCommand::AddAudioTrack));
    assert_eq!(app.mixer.channels.len(), 3);
    assert_eq!(app.mixer.channels[2].name, "Audio 3");

    // 2. Add MIDI Track via MenuAction
    app.update(UiMessage::MenuAction(MenuCommand::AddMidiTrack));
    assert_eq!(app.mixer.channels.len(), 4);
    assert_eq!(app.mixer.channels[3].name, "Inst 4");

    // 3. Toggle Mixer Drawer via MenuAction
    app.update(UiMessage::MenuAction(MenuCommand::ToggleMixer));
    assert!(app.workspace.mixer_drawer_open);

    // 4. Toggle Inspector via MenuAction (closes it)
    app.update(UiMessage::MenuAction(MenuCommand::ToggleInspector));
    assert!(!app.workspace.inspector_open);

    // 5. Toggle Piano Roll via MenuAction
    app.update(UiMessage::MenuAction(MenuCommand::TogglePianoRoll));
    assert!(app.workspace.editor_drawer_open);
    assert_eq!(app.workspace.active_editor, EditorDrawerTab::PianoRoll);

    // 6. Test Channel Strip Fader adjustment from Drawer
    app.update(UiMessage::SetTrackFader {
        track_id: 0,
        volume_db: -3.0,
    });
    assert_eq!(app.mixer.channels[0].volume_fader_db, -3.0);

    // 7. Test Mute and Solo
    app.update(UiMessage::ToggleTrackMute(0));
    assert!(app.mixer.channels[0].muted);
    app.update(UiMessage::ToggleTrackSolo(0));
    assert!(app.mixer.channels[0].solo);
}

#[test]
fn test_ui_view_rendering_with_active_drawers() {
    let mut app = CrystalWaveApp::new(44100.0);
    app.workspace.inspector_open = true;
    app.workspace.mixer_drawer_open = true;

    // Call view() to ensure widget tree compiles and renders without panicking
    let _element = app.view();
}
