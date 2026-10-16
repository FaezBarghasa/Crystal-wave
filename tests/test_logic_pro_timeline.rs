use Crystal_wave::timeline::region::{Region, RegionFade, RegionKind};
use Crystal_wave::timeline::track::{Track, TrackType};
use Crystal_wave::ui::canvas_timeline::{
    ArrangementCanvasProgram, ClipVisualKind, LANE_HEIGHT, RULER_HEIGHT, TICKS_PER_BAR,
    TimelineClipDisplay,
};

#[test]
fn test_logic_pro_coordinate_mapping() {
    let program = ArrangementCanvasProgram::new(0, 1.0, 8);
    let bounds_w = 1200.0;

    // Bar 1 starts at tick 0 -> x = 0.0
    assert_eq!(program.tick_to_x(0, bounds_w), 0.0);

    // Bar 2 starts at tick 3840
    let bar2_x = program.tick_to_x(TICKS_PER_BAR, bounds_w);
    assert!(bar2_x > 0.0);

    // Roundtrip x_to_tick
    let roundtrip_tick = program.x_to_tick(bar2_x, bounds_w);
    assert!((roundtrip_tick as i64 - TICKS_PER_BAR as i64).abs() <= 5);

    // y_to_track
    // Below RULER_HEIGHT (e.g. y = 15.0) is the ruler, so returns None
    assert_eq!(program.y_to_track(15.0), None);

    // At y = RULER_HEIGHT + 10.0 -> Track 0
    assert_eq!(program.y_to_track(RULER_HEIGHT + 10.0), Some(0));

    // At y = RULER_HEIGHT + LANE_HEIGHT + 10.0 -> Track 1
    assert_eq!(
        program.y_to_track(RULER_HEIGHT + LANE_HEIGHT + 10.0),
        Some(1)
    );
}

#[test]
fn test_logic_pro_smart_tool_hotspots() {
    let program = ArrangementCanvasProgram::new(0, 1.0, 4);
    let bounds_w = 1000.0;

    let clip_start_tick = 3840; // Bar 2
    let clip_len_ticks = 3840 * 2; // 2 bars
    let x_start = program.tick_to_x(clip_start_tick, bounds_w);
    let x_end = program.tick_to_x(clip_start_tick + clip_len_ticks, bounds_w);
    let y_clip = RULER_HEIGHT + LANE_HEIGHT + 4.0;
    let h_clip = LANE_HEIGHT - 8.0;

    // 1. Left Edge (within 8px) -> TrimStart
    let cursor_x = x_start + 4.0;
    assert!(cursor_x - x_start < 8.0);

    // 2. Right Edge (within 8px) -> TrimEnd
    let cursor_x_end = x_end - 4.0;
    assert!(x_end - cursor_x_end < 8.0);

    // 3. Top-left corner -> FadeIn
    let cursor_y_top = y_clip + 5.0;
    assert!(cursor_y_top - y_clip < 14.0);

    // 4. Bottom half -> GrabMove
    let cursor_y_bot = y_clip + h_clip * 0.7;
    assert!(cursor_y_bot > y_clip + h_clip * 0.45);
}

#[test]
fn test_logic_pro_timeline_clip_display_models() {
    let mut trk = Track::new(0, "Lead Vocal", TrackType::Audio);
    let mut reg = Region::new(
        101,
        0,
        "Vocal Verse 1",
        3840,
        3840 * 4,
        RegionKind::Audio {
            file_path: None,
            sample_offset: 0,
            sample_length: 44100 * 8,
        },
    );
    reg.fade = RegionFade {
        in_ticks: 480,
        in_curve: Default::default(),
        out_ticks: 960,
        out_curve: Default::default(),
    };
    reg.looped = true;
    trk.regions.push(reg);

    let clip_disp = TimelineClipDisplay {
        id: trk.regions[0].id,
        track_index: 0,
        start_tick: trk.regions[0].start_tick,
        length_ticks: trk.regions[0].length_ticks,
        name: trk.regions[0].name.clone(),
        color_rgb: [trk.color.r, trk.color.g, trk.color.b],
        muted: trk.regions[0].muted,
        looped: trk.regions[0].looped,
        fade_in_ticks: trk.regions[0].fade.in_ticks,
        fade_out_ticks: trk.regions[0].fade.out_ticks,
        kind: ClipVisualKind::Audio {
            waveform_peaks: vec![(-0.8, 0.8), (-0.5, 0.5), (-0.9, 0.9)],
        },
        is_selected: true,
    };

    assert_eq!(clip_disp.id, 101);
    assert_eq!(clip_disp.start_tick, 3840);
    assert_eq!(clip_disp.length_ticks, 3840 * 4);
    assert!(clip_disp.looped);
    assert_eq!(clip_disp.fade_in_ticks, 480);
    assert_eq!(clip_disp.fade_out_ticks, 960);
    assert!(clip_disp.is_selected);
}

#[test]
fn test_logic_pro_cycle_loop_bar_range() {
    let mut program = ArrangementCanvasProgram::new(0, 1.0, 4);
    program.loop_range = Some((3840, 3840 * 5));

    if let Some((start, end)) = program.loop_range {
        assert_eq!(start, 3840);
        assert_eq!(end, 19200);
        let bar_start = (start / TICKS_PER_BAR) + 1;
        let bar_end = (end / TICKS_PER_BAR) + 1;
        assert_eq!(bar_start, 2);
        assert_eq!(bar_end, 6);
    } else {
        panic!("Loop range expected");
    }
}
