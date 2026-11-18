use Crystal_wave::ui::app::CrystalWaveApp;
use Crystal_wave::ui::message::{PianoRollTool, PlaylistTool, UiMessage};

#[test]
fn test_fl_studio_transport_pat_song_mode() {
    let mut app = CrystalWaveApp::new(44100.0);
    assert!(app.is_song_mode, "Default mode is SONG mode (true)");

    app.update(UiMessage::TogglePatternSongMode);
    assert!(!app.is_song_mode, "Should switch to PAT mode (false)");

    app.update(UiMessage::TogglePatternSongMode);
    assert!(app.is_song_mode, "Should toggle back to SONG mode (true)");
}

#[test]
fn test_fl_studio_pattern_selection() {
    let mut app = CrystalWaveApp::new(44100.0);
    assert_eq!(app.current_pattern_index, 1);

    app.update(UiMessage::SelectPattern(4));
    assert_eq!(app.current_pattern_index, 4);

    app.update(UiMessage::SelectPattern(99));
    assert_eq!(app.current_pattern_index, 99);
}

#[test]
fn test_fl_studio_performance_toggles() {
    let mut app = CrystalWaveApp::new(44100.0);
    assert!(!app.metronome_enabled);
    assert!(!app.precount_enabled);
    assert!(app.typing_keyboard_to_piano);
    assert!(app.blend_recording_enabled);

    app.update(UiMessage::ToggleMetronome);
    assert!(app.metronome_enabled);

    app.update(UiMessage::TogglePrecount);
    assert!(app.precount_enabled);

    app.update(UiMessage::ToggleTypingKeyboard);
    assert!(!app.typing_keyboard_to_piano);

    app.update(UiMessage::ToggleBlendRecording);
    assert!(!app.blend_recording_enabled);

    // Toggle back
    app.update(UiMessage::ToggleMetronome);
    assert!(!app.metronome_enabled);
}

#[test]
fn test_fl_studio_master_pitch_and_volume() {
    let mut app = CrystalWaveApp::new(44100.0);
    assert!((app.master_volume - 1.0).abs() < 1e-4);
    assert_eq!(app.master_pitch_cents, 0);

    app.update(UiMessage::SetMasterVolume(0.85));
    assert!((app.master_volume - 0.85).abs() < 1e-4);

    app.update(UiMessage::SetMasterPitchCents(-300));
    assert_eq!(app.master_pitch_cents, -300);

    // Clamping limits
    app.update(UiMessage::SetMasterVolume(2.0));
    assert!((app.master_volume - 1.25).abs() < 1e-4);

    app.update(UiMessage::SetMasterPitchCents(1500));
    assert_eq!(app.master_pitch_cents, 1200);
}

#[test]
fn test_fl_studio_fruity_wrapper_channel_settings() {
    let mut app = CrystalWaveApp::new(44100.0);
    assert!(app.active_channel_settings.is_none());

    // Open channel 0 settings
    app.update(UiMessage::OpenChannelSettings(0));
    assert_eq!(app.active_channel_settings, Some(0));

    // Modify pitch
    app.update(UiMessage::ChannelSettingsSetPitch {
        channel_id: 0,
        pitch_cents: 200,
    });
    if let Some(ch) = app.channel_rack.channels.first() {
        assert_eq!(ch.pitch_semitones, 2);
    }

    // Modify pan
    app.update(UiMessage::ChannelSettingsSetPan {
        channel_id: 0,
        pan: -0.65,
    });
    if let Some(ch) = app.channel_rack.channels.first() {
        assert!((ch.pan - (-0.65)).abs() < 1e-4);
    }

    // Modify volume
    app.update(UiMessage::ChannelSettingsSetVolume {
        channel_id: 0,
        volume: 0.95,
    });
    if let Some(ch) = app.channel_rack.channels.first() {
        assert!((ch.volume - 0.95).abs() < 1e-4);
    }

    // Modify mixer track target
    app.update(UiMessage::ChannelSettingsSetMixerTrack {
        channel_id: 0,
        track: 12,
    });
    if let Some(ch) = app.channel_rack.channels.first() {
        assert_eq!(ch.target_mixer_track, 12);
    }

    // Close settings modal
    app.update(UiMessage::CloseChannelSettings);
    assert!(app.active_channel_settings.is_none());
}

#[test]
fn test_fl_studio_channel_rack_features() {
    let mut app = CrystalWaveApp::new(44100.0);
    assert_eq!(app.channel_rack_step_count, 16);
    assert!(!app.graph_editor_open);

    app.update(UiMessage::ToggleChannelRackGraphEditor);
    assert!(app.graph_editor_open);

    app.update(UiMessage::SetChannelRackStepCount(32));
    assert_eq!(app.channel_rack_step_count, 32);

    app.update(UiMessage::SetChannelRackStepCount(64));
    assert_eq!(app.channel_rack_step_count, 64);
}

#[test]
fn test_fl_studio_tool_palettes() {
    let mut app = CrystalWaveApp::new(44100.0);
    assert_eq!(app.active_playlist_tool, PlaylistTool::Draw);
    assert_eq!(app.active_pianoroll_tool, PianoRollTool::Draw);

    app.update(UiMessage::SetPlaylistTool(PlaylistTool::Slice));
    assert_eq!(app.active_playlist_tool, PlaylistTool::Slice);

    app.update(UiMessage::SetPlaylistTool(PlaylistTool::Paint));
    assert_eq!(app.active_playlist_tool, PlaylistTool::Paint);

    app.update(UiMessage::SetPianoRollTool(PianoRollTool::Select));
    assert_eq!(app.active_pianoroll_tool, PianoRollTool::Select);

    app.update(UiMessage::SetPianoRollTool(PianoRollTool::Delete));
    assert_eq!(app.active_pianoroll_tool, PianoRollTool::Delete);
}

#[test]
fn test_fl_studio_full_view_render_stability() {
    let mut app = CrystalWaveApp::new(44100.0);

    // Initial state render
    {
        let _ = app.view();
    }

    // Render with Fruity Wrapper modal active
    app.active_channel_settings = Some(0);
    {
        let _ = app.view();
    }

    // Render with Graph Editor open
    app.graph_editor_open = true;
    {
        let _ = app.view();
    }

    // Render with Song mode active
    app.is_song_mode = true;
    {
        let _ = app.view();
    }

    // Render with all FL windows toggled
    app.fl_windows.playlist_visible = true;
    app.fl_windows.channel_rack_visible = true;
    app.fl_windows.piano_roll_visible = true;
    app.fl_windows.mixer_visible = true;
    app.fl_windows.browser_visible = true;
    app.fl_windows.plugin_picker_visible = true;
    app.audio_settings_open = true;
    app.command_palette_open = true;

    {
        let _ = app.view();
    }
}
