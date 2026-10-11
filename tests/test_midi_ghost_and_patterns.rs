use Crystal_wave::midi::{GhostNote, GhostPolyphonyView};
use Crystal_wave::mixing::{AutomationShape, FloatingAutomationObject};
use Crystal_wave::timeline::{HybridTimelineManager, PatternNote};

#[test]
fn test_ghost_polyphony_and_microtonal_notes() {
    let mut ghost_view = GhostPolyphonyView::new(1); // Active on track 1 (Lead Synth)
    assert_eq!(ghost_view.active_track_id, 1);

    // Notes from all tracks: (track_id, pitch, start_tick, len_ticks, vel, color)
    let all_notes = vec![
        (0, 36, 0, 960, 100, [50, 100, 200]),  // Track 0 (Bass): C2
        (1, 60, 0, 480, 110, [255, 100, 50]),  // Track 1 (Active Synth): C4
        (2, 64, 480, 480, 90, [100, 255, 100]), // Track 2 (Pad): E4
    ];

    ghost_view.update_ghosts(&all_notes);

    // Ghost view should project Track 0 and Track 2 notes, excluding active Track 1
    assert_eq!(ghost_view.ghost_notes.len(), 2);
    assert_eq!(ghost_view.ghost_notes[0].source_track_id, 0);
    assert_eq!(ghost_view.ghost_notes[0].pitch, 36);
    assert_eq!(ghost_view.ghost_notes[1].source_track_id, 2);
    assert_eq!(ghost_view.ghost_notes[1].pitch, 64);

    // Active ghost notes at tick 200 (Bass note only)
    let ghosts_at_200 = ghost_view.active_ghosts_at(200);
    assert_eq!(ghosts_at_200.len(), 1);
    assert_eq!(ghosts_at_200[0].pitch, 36);

    // Microtonal slide note
    let slide_ghost = GhostNote::new(0, 48, 0, 960, 100, [0, 0, 0])
        .with_slide(true)
        .with_microtonal(35.5);
    assert!(slide_ghost.is_slide);
    assert!((slide_ghost.microtonal_cents - 35.5).abs() < 1e-4);
}

#[test]
fn test_hybrid_pattern_timeline_and_linear_explode() {
    let mut hybrid = HybridTimelineManager::new();

    // 1. Create drum core pattern (4 beats = 960 * 4 ticks)
    let pat_id = hybrid.create_pattern("Trap 808 & Snare Loop", 960 * 4);

    if let Some(pat) = hybrid.patterns.get_mut(&pat_id) {
        pat.add_note(36, 0, 240, 120);     // Kick
        pat.add_note(38, 960, 240, 110);    // Snare
        pat.add_note(36, 960 * 2, 240, 115); // Kick
        pat.add_note(38, 960 * 3, 240, 110); // Snare
    }

    // 2. Stamp pattern at measure 1, measure 5, measure 9
    let inst1 = hybrid.stamp_pattern(pat_id, 0, 0).expect("Stamp 1");
    let inst2 = hybrid.stamp_pattern(pat_id, 0, 960 * 4).expect("Stamp 2");
    let inst3 = hybrid.stamp_pattern(pat_id, 0, 960 * 8).expect("Stamp 3");

    assert_eq!(hybrid.linked_instance_count(pat_id), 3);

    // 3. Explode / detach instance 3 to create an independent drum fill
    let exploded_pat_id = hybrid
        .explode_instance(inst3)
        .expect("Explode instance 3");

    assert_eq!(hybrid.linked_instance_count(pat_id), 2);
    assert_eq!(hybrid.linked_instance_count(exploded_pat_id), 0); // Detached

    // Edit the exploded pattern linearly without altering original instances 1 & 2
    if let Some(exploded_pat) = hybrid.patterns.get_mut(&exploded_pat_id) {
        exploded_pat.add_note(42, 960 * 3 + 480, 120, 127); // Add fill roll
        assert_eq!(exploded_pat.notes.len(), 5);
    }

    // Original pattern still has only 4 notes
    assert_eq!(hybrid.patterns.get(&pat_id).unwrap().notes.len(), 4);
    assert!(!hybrid.instances.iter().find(|i| i.instance_id == inst1).unwrap().is_detached);
    assert!(hybrid.instances.iter().find(|i| i.instance_id == inst3).unwrap().is_detached);
}

#[test]
fn test_free_form_floating_automation_objects() {
    let mut auto_obj = FloatingAutomationObject::new(1, "Filter Sweep", 960 * 4, AutomationShape::Spline);
    auto_obj.add_node(960 * 2, 0.25, 0.5); // Midpoint curve with tension

    // Initial evaluation
    let start_val = auto_obj.evaluate(0);
    assert!((start_val - 0.0).abs() < 1e-4);

    let end_val = auto_obj.evaluate(960 * 4);
    assert!((end_val - 1.0).abs() < 1e-4);

    // Stretch automation object across entire chorus (960 * 16 ticks)
    auto_obj.stretch_to(960 * 16);
    assert_eq!(auto_obj.length_ticks, 960 * 16);
    assert_eq!(auto_obj.nodes.last().unwrap().tick_offset, 960 * 16);

    // Drag-and-drop onto track 2 "Cutoff" parameter
    auto_obj.drop_onto_track(2, "Cutoff");
    assert_eq!(auto_obj.assigned_track_id, Some(2));
    assert_eq!(auto_obj.assigned_param_name.as_deref(), Some("Cutoff"));
}
