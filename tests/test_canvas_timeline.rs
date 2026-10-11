use Crystal_wave::ui::canvas_timeline::ArrangementCanvasProgram;

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
