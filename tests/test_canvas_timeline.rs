use Crystal_wave::timeline::region::{AutomationPoint, Region, RegionKind};
use Crystal_wave::timeline::track::TrackType;
use Crystal_wave::ui::app::CrystalWaveApp;
use Crystal_wave::ui::canvas_timeline::{ArrangementCanvasProgram, evaluate_spline_tension};
use Crystal_wave::ui::message::UiMessage;
use Crystal_wave::ui::theme::ThemePreset;

#[test]
fn test_canvas_timeline_coordinate_conversions() {
    let program = ArrangementCanvasProgram::new(0, 1.0, 4);

    let bounds_width = 1000.0;

    // Tick 0 -> x = 0.0
    let x0 = program.tick_to_x(0, bounds_width);
    assert_eq!(x0, 0.0);

    // End tick -> x = bounds_width
    let x_end = program.tick_to_x(program.visible_end_tick, bounds_width);
    assert!((x_end - bounds_width).abs() < 1e-3);

    // Inverse x -> tick
    let tick_mid = program.x_to_tick(bounds_width / 2.0, bounds_width);
    let expected_mid = program.visible_end_tick / 2;
    assert!((tick_mid as i64 - expected_mid as i64).abs() <= 1);
}

#[test]
fn test_canvas_timeline_zoom_scaling() {
    let mut program = ArrangementCanvasProgram::new(0, 2.0, 4);
    let bounds_width = 800.0;

    // At 2.0x horizontal zoom, tick midpoint appears at bounds_width
    let mid_tick = program.visible_end_tick / 2;
    let x_mid = program.tick_to_x(mid_tick, bounds_width);
    assert!((x_mid - bounds_width).abs() < 2.0);

    // Test clamped extreme zooms
    program.horizontal_zoom = 50.0;
    let tick = program.x_to_tick(bounds_width, bounds_width);
    assert!(tick <= program.visible_end_tick);
}

#[test]
fn test_hermite_spline_tension_evaluation() {
    // 1. Boundaries at u=0.0 and u=1.0 must be exact across all tensions
    for &tension in &[-1.0, -0.5, 0.0, 0.5, 1.0] {
        assert!((evaluate_spline_tension(0.0, tension) - 0.0).abs() < 1e-5);
        assert!((evaluate_spline_tension(1.0, tension) - 1.0).abs() < 1e-5);
    }

    // 2. Linear tension (T = 0.0) -> midpoint u = 0.5 yields 0.5
    let lin_mid = evaluate_spline_tension(0.5, 0.0);
    assert!((lin_mid - 0.5).abs() < 1e-5);

    // 3. Convex tension (T = 0.5 > 0.0) -> bows downward / slow start
    // progress = u^(1 + 0.5*3) = 0.5^2.5 ≈ 0.1768 < 0.5
    let convex_mid = evaluate_spline_tension(0.5, 0.5);
    assert!(
        convex_mid < 0.5,
        "Convex tension must curve below linear diagonal"
    );

    // 4. Concave tension (T = -0.5 < 0.0) -> bows upward / fast start
    // progress = 1 - (1 - 0.5)^(1 - (-0.5)*3) = 1 - 0.5^2.5 ≈ 0.8232 > 0.5
    let concave_mid = evaluate_spline_tension(0.5, -0.5);
    assert!(
        concave_mid > 0.5,
        "Concave tension must curve above linear diagonal"
    );
}

#[test]
fn test_region_automation_curve_evaluation() {
    let points = vec![
        AutomationPoint::new(0, 0.0, 0.5), // start at 0% with convex tension
        AutomationPoint::new(1920, 1.0, 0.0), // peak at 100% linear
        AutomationPoint::new(3840, 0.2, -0.5), // drop to 20% with concave tension
    ];

    let region = Region {
        id: 101,
        track_id: 0,
        name: "Filter Cutoff Auto".to_string(),
        start_tick: 0,
        length_ticks: 3840,
        anchor_offset_ticks: 0,
        muted: false,
        looped: false,
        loop_length_ticks: None,
        gain_db: 0.0,
        transpose_semitones: 0.0,
        reversed: false,
        fade: Default::default(),
        kind: RegionKind::Automation {
            parameter_target: "Filter Cutoff".to_string(),
            min_value: 20.0,
            max_value: 20000.0,
            points,
        },
    };

    // At offset 0 -> min_value + (max - min)*0.0 = 20.0 Hz
    let val_0 = region.evaluate_automation(0).expect("Must return value");
    assert!((val_0 - 20.0).abs() < 1e-3);

    // At offset 1920 -> min_value + (max - min)*1.0 = 20000.0 Hz
    let val_1920 = region.evaluate_automation(1920).expect("Must return value");
    assert!((val_1920 - 20000.0).abs() < 1e-3);

    // At offset 3840 -> min_value + (max - min)*0.2 = 20 + 19980*0.2 = 4016.0 Hz
    let val_3840 = region.evaluate_automation(3840).expect("Must return value");
    assert!((val_3840 - 4016.0).abs() < 1.0);

    // Midpoint between 0 and 1920 (tick 960): convex curve should be below linear mid
    let val_960 = region.evaluate_automation(960).expect("Must return value");
    let linear_mid = 20.0 + (20000.0 - 20.0) * 0.5;
    assert!(
        val_960 < linear_mid,
        "Convex tension should curve below linear midpoint"
    );
}

#[test]
fn test_free_form_track_coexistence_and_interaction() {
    let mut app = CrystalWaveApp::new(44100.0);
    app.update(UiMessage::SetThemePreset(ThemePreset::FlStudioModern));

    // Ensure we have at least 2 tracks (Track 0: Audio, Track 1: Software)
    assert!(app.arrangement.timeline.tracks.len() >= 2);
    app.arrangement.timeline.tracks[0].track_type = TrackType::Audio;
    app.arrangement.timeline.tracks[1].track_type = TrackType::Software;

    // 1. Add Automation Clip to Track 0 (Audio track)
    app.update(UiMessage::TimelineAddAutomationClip {
        track_id: 0,
        start_tick: 0,
        length_ticks: 3840,
        parameter_target: "Master Volume".to_string(),
    });

    let auto_clip_id = app.arrangement.timeline.tracks[0]
        .regions
        .last()
        .expect("Clip must exist")
        .id;

    // 2. Add Pattern Clip to Track 0 (Free-form coexistence with Audio & Automation on same track!)
    app.update(UiMessage::TimelineAddPatternClip {
        track_id: 0,
        start_tick: 3840,
        length_ticks: 3840,
        name: "Pattern 1".to_string(),
    });

    let pattern_clip_id = app.arrangement.timeline.tracks[0]
        .regions
        .last()
        .expect("Pattern clip must exist")
        .id;

    // Verify both clips reside in Track 0
    let trk0_regions = &app.arrangement.timeline.tracks[0].regions;
    assert!(
        trk0_regions
            .iter()
            .any(|r| matches!(r.kind, RegionKind::Automation { .. }))
    );
    assert!(
        trk0_regions
            .iter()
            .any(|r| matches!(r.kind, RegionKind::Pattern { .. }))
    );

    // 3. Move the Pattern Clip from Track 0 (Audio track) to Track 1 (Software track)
    app.update(UiMessage::TimelineMoveClip {
        clip_id: pattern_clip_id,
        new_track_id: 1,
        new_start_tick: 7680,
    });

    assert!(
        app.arrangement.timeline.tracks[1]
            .regions
            .iter()
            .any(|r| r.id == pattern_clip_id)
    );

    // 4. Test interactive tension adjusting on the automation clip
    app.update(UiMessage::TimelineSetAutomationTension {
        clip_id: auto_clip_id,
        segment_index: 0,
        tension: 0.75,
    });

    // 5. Test interactive node value adjusting on the automation clip
    app.update(UiMessage::TimelineSetAutomationNode {
        clip_id: auto_clip_id,
        node_index: 1,
        value: 0.90,
    });

    // Verify updated values in the model
    let auto_reg = app.arrangement.timeline.tracks[0]
        .regions
        .iter()
        .find(|r| r.id == auto_clip_id)
        .expect("Automation clip must be present");

    if let RegionKind::Automation { points, .. } = &auto_reg.kind {
        assert_eq!(points[0].tension, 0.75);
        assert_eq!(points[1].value, 0.90);
    } else {
        panic!("Region must be Automation");
    }
}
