use Crystal_wave::timeline::arrangement_view::CatchPlayheadMode;
use Crystal_wave::ui::message::UiMessage;
use Crystal_wave::ui::session_driver::UiSessionDriver;
use Crystal_wave::ui::telemetry::TelemetryChannel;
use Crystal_wave::ui_state::EditorDrawerTab;

#[test]
fn test_keyboard_shortcut_drawer_toggling() {
    let mut driver = UiSessionDriver::new(44100.0);

    // Initial state: inspector open by default, drawers closed
    driver.assert_inspector_open(true);
    driver.assert_editor_open(false);
    driver.assert_mixer_open(false);

    // Press 'I' to toggle inspector off
    driver.dispatch_shortcut('i');
    driver.assert_inspector_open(false);

    // Press 'I' to toggle inspector on
    driver.dispatch_shortcut('I');
    driver.assert_inspector_open(true);

    // Press 'E' to toggle editor drawer on
    driver.dispatch_shortcut('e');
    driver.assert_editor_open(true);

    // Press 'X' to toggle mixer drawer on
    driver.dispatch_shortcut('x');
    driver.assert_mixer_open(true);
}

#[test]
fn test_transport_and_playhead_controls() {
    let mut driver = UiSessionDriver::new(44100.0);

    driver.assert_playing(false);
    driver.assert_recording(false);
    driver.assert_playhead(0);

    // Spacebar toggles play
    driver.dispatch_shortcut(' ');
    driver.assert_playing(true);
    driver.assert_recording(false);

    // Spacebar toggles pause
    driver.dispatch_shortcut(' ');
    driver.assert_playing(false);

    // 'R' starts recording
    driver.dispatch_shortcut('r');
    driver.assert_playing(true);
    driver.assert_recording(true);

    // Enter stops and resets playhead
    driver.dispatch(UiMessage::SetPlayhead(3840));
    driver.assert_playhead(3840);
    driver.dispatch_shortcut('\n');
    driver.assert_playing(false);
    driver.assert_recording(false);
    driver.assert_playhead(0);
}

#[test]
fn test_full_user_session_workflow() {
    let mut driver = UiSessionDriver::new(44100.0);

    // 1. Select Track & Adjust Fader / Pan
    driver
        .dispatch(UiMessage::SelectTrack(1))
        .dispatch(UiMessage::SetTrackFader {
            track_id: 1,
            volume_db: -6.0,
        })
        .dispatch(UiMessage::SetTrackPan {
            track_id: 1,
            pan: 0.5,
        });

    assert_eq!(driver.app.workspace.selected_track_id, 1);
    assert_eq!(driver.app.inspector.selected_track_id, 1);
    assert_eq!(driver.app.mixer.channels[1].volume_fader_db, -6.0);
    assert_eq!(driver.app.mixer.channels[1].pan.balance, 0.5);

    // 2. Open Piano Roll editor tab
    driver.dispatch(UiMessage::SelectEditorTab(EditorDrawerTab::PianoRoll));
    driver.assert_editor_open(true);
    assert_eq!(
        driver.app.workspace.active_editor,
        EditorDrawerTab::PianoRoll
    );

    // 3. Set Timeline zoom and catch playhead
    driver
        .dispatch(UiMessage::SetHorizontalZoom(2.5))
        .dispatch(UiMessage::SetVerticalZoom(1.5))
        .dispatch(UiMessage::SetCatchPlayhead(
            CatchPlayheadMode::ContinuousScroll,
        ));

    assert_eq!(driver.app.arrangement.horizontal_zoom, 2.5);
    assert_eq!(driver.app.arrangement.vertical_zoom, 1.5);
    assert_eq!(
        driver.app.arrangement.catch_playhead,
        CatchPlayheadMode::ContinuousScroll
    );
}

#[test]
fn test_render_element_tree_headless() {
    let driver = UiSessionDriver::new(44100.0);

    // Verify view() generates clean declarative widget tree without panic
    let _element = driver.app.view();
}

#[test]
fn test_telemetry_ring_buffer_channel() {
    let (mut producer, mut consumer) = TelemetryChannel::new(16);

    let batch = Crystal_wave::ui::telemetry::TelemetryBatch {
        playhead_tick: 960,
        cpu_load_percent: 12.5,
        meters: vec![Crystal_wave::ui::telemetry::ChannelMeterFrame {
            channel_id: 0,
            peak_db: -3.0,
            rms_db: -12.0,
            auto_mute_tripped: false,
        }],
    };

    assert!(producer.push(batch.clone()).is_ok());

    let received = consumer.pop().expect("pop batch");
    assert_eq!(received.playhead_tick, 960);
    assert_eq!(received.cpu_load_percent, 12.5);
    assert_eq!(received.meters[0].peak_db, -3.0);
}
