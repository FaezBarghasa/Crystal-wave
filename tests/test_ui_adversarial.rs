use Crystal_wave::ui::message::UiMessage;
use Crystal_wave::ui::session_driver::UiSessionDriver;

#[test]
fn test_angry_user_fader_slamming_chaos() {
    let mut driver = UiSessionDriver::new(44100.0);

    // Chaos simulation: 10,000 rapid chaotic fader jumps from -120 to +12 dB
    for i in 0..10_000 {
        let val = if i % 2 == 0 {
            -120.0 + (i as f32 % 130.0)
        } else {
            12.0 - (i as f32 % 140.0)
        };
        driver.dispatch(UiMessage::SetTrackFader {
            track_id: 0,
            volume_db: val,
        });
    }

    // Fader must be clamped within [-120.0, 6.0]
    let fader = driver.app.mixer.channels[0].volume_fader_db;
    assert!(fader >= -120.0 && fader <= 6.0);
}

#[test]
fn test_poison_values_injection() {
    let mut driver = UiSessionDriver::new(44100.0);

    // 1. Inject NaNs into fader
    driver.dispatch(UiMessage::SetTrackFader {
        track_id: 0,
        volume_db: f32::NAN,
    });
    assert_eq!(driver.app.mixer.channels[0].volume_fader_db, -120.0);

    // 2. Inject +Infinity and -Infinity into pan
    driver.dispatch(UiMessage::SetTrackPan {
        track_id: 0,
        pan: f32::INFINITY,
    });
    assert_eq!(driver.app.mixer.channels[0].pan.balance, 0.0);

    driver.dispatch(UiMessage::SetTrackPan {
        track_id: 0,
        pan: f32::NEG_INFINITY,
    });
    assert_eq!(driver.app.mixer.channels[0].pan.balance, 0.0);

    // 3. Inject negative and non-finite tempo
    driver.dispatch(UiMessage::SetTempoBpm(-500.0));
    assert_eq!(
        driver.app.workspace.lcd_diagnostic.project_tempo_bpm,
        10.0
    );

    driver.dispatch(UiMessage::SetTempoBpm(f32::NAN));
    assert_eq!(
        driver.app.workspace.lcd_diagnostic.project_tempo_bpm,
        120.0
    );

    // 4. Inject extreme zoom
    driver.dispatch(UiMessage::SetHorizontalZoom(-100.0));
    assert_eq!(driver.app.arrangement.horizontal_zoom, 0.1);

    driver.dispatch(UiMessage::SetHorizontalZoom(9999.0));
    assert_eq!(driver.app.arrangement.horizontal_zoom, 50.0);
}

#[test]
fn test_out_of_bounds_track_access() {
    let mut driver = UiSessionDriver::new(44100.0);

    // Dispatching out-of-bounds track ID must not panic
    driver.dispatch(UiMessage::SelectTrack(999_999));
    driver.dispatch(UiMessage::SetTrackFader {
        track_id: 888_888,
        volume_db: -3.0,
    });
    driver.dispatch(UiMessage::ToggleTrackMute(777_777));
    driver.dispatch(UiMessage::ToggleTrackSolo(666_666));
    driver.dispatch(UiMessage::ResetAutoMute(555_555));
}

#[test]
fn test_auto_mute_protection_trip_and_ui_recovery() {
    let mut driver = UiSessionDriver::new(44100.0);

    // Simulate extreme overload tripping auto-mute on channel 0
    let mut left = vec![10.0; 256];
    let mut right = vec![10.0; 256];
    driver.app.mixer.channels[0]
        .meter
        .process_block(&mut left, &mut right);

    assert!(driver.app.mixer.channels[0].meter.auto_mute_tripped);
    assert_eq!(left[0], 0.0); // Muted by safety limiter

    // Safe recovery via UI action
    driver.dispatch(UiMessage::ResetAutoMute(0));
    assert!(!driver.app.mixer.channels[0].meter.auto_mute_tripped);
}
