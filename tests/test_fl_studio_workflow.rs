use Crystal_wave::fl_studio::{
    automation_clip::{AutomationClip, SplineNode},
    browser::{FlBrowser, SampleEntry},
    channel_rack::{ChannelRack, StepProperties},
    gross_beat::{GrossBeatEngine, GrossBeatPreset},
    history::FlHistoryManager,
    patcher::{FlPatcher, PatcherCable, PatcherMacroKnob, PatcherNode},
    piano_roll::{FlPianoRollNote, FlPianoRollTools, GhostNote},
    playlist::{FlPattern, FlPlaylist, PlaylistItem, PlaylistTrackLane},
    soft_clipper::FruitySoftClipper,
    FlWindowKind, FlWindowManager,
};

#[test]
fn test_piano_roll_slide_notes_and_tools() {
    let mut notes = vec![
        FlPianoRollNote {
            pitch: 60,
            start_tick: 0,
            duration_ticks: 96,
            velocity: 0.8,
            pan: 0.0,
            is_slide: false,
            slide_target_pitch: None,
        },
        FlPianoRollNote {
            pitch: 64,
            start_tick: 96,
            duration_ticks: 96,
            velocity: 0.8,
            pan: 0.0,
            is_slide: true,
            slide_target_pitch: Some(72),
        },
    ];

    assert!(notes[1].is_slide);
    assert_eq!(notes[1].slide_target_pitch, Some(72));

    // Strum tool
    FlPianoRollTools::quick_strum(&mut notes, 12, 0.05);
    assert_eq!(notes[0].start_tick, 0);
    assert_eq!(notes[1].start_tick, 108);

    // Flip horizontal
    FlPianoRollTools::flip_horizontal(&mut notes, 204);
    assert_eq!(notes[1].start_tick, 0);

    // Flip vertical
    FlPianoRollTools::flip_vertical(&mut notes, 60);
    assert_eq!(notes[0].pitch, 60);

    // Chop
    FlPianoRollTools::quick_chop(&mut notes, 48);
    assert!(notes.len() >= 4);

    // Ghost notes representation
    let ghost = GhostNote {
        pitch: 67,
        start_tick: 0,
        duration_ticks: 192,
        velocity: 0.7,
        source_pattern_id: 2,
    };
    assert_eq!(ghost.source_pattern_id, 2);
}

#[test]
fn test_channel_rack_and_graph_editor() {
    let mut rack = ChannelRack::new(16);
    let ch_idx = rack.add_channel("808 Bass", 36);
    assert_eq!(ch_idx, 0);

    // Toggle steps
    rack.toggle_step(0, 0);
    rack.toggle_step(0, 4);
    rack.toggle_step(0, 8);
    rack.toggle_step(0, 12);

    assert!(rack.channels[0].steps[0]);
    assert!(rack.channels[0].steps[4]);
    assert!(!rack.channels[0].steps[1]);

    // Graph editor per-step properties
    rack.channels[0].graph_properties[4] = StepProperties {
        velocity: 0.95,
        pan: -0.2,
        pitch_offset: 12,
        shift_ticks: 3,
    };

    assert_eq!(rack.channels[0].graph_properties[4].pitch_offset, 12);
    assert_eq!(rack.channels[0].graph_properties[4].velocity, 0.95);

    // Export to MIDI note events
    let midi_notes = rack.to_midi_notes(0, 96);
    assert_eq!(midi_notes.len(), 4);
    assert_eq!(midi_notes[0].pitch, 36);
    assert_eq!(midi_notes[1].pitch, 48); // Pitch offset +12 applied
}

#[test]
fn test_playlist_free_form_coexistence() {
    let mut playlist = FlPlaylist::new();
    let lane_idx = playlist.add_lane("Main Lane");

    let mut pattern = FlPattern::new(1, "Trap Drums", [1.0, 0.2, 0.4, 1.0]);
    pattern.add_note(FlPianoRollNote {
        pitch: 36,
        start_tick: 0,
        duration_ticks: 96,
        velocity: 0.9,
        pan: 0.0,
        is_slide: false,
        slide_target_pitch: None,
    });
    playlist.add_pattern(pattern);

    // Place Pattern, Audio Clip, and Automation Clip all in the same lane
    playlist.lanes[lane_idx].items.push(PlaylistItem::PatternInstance {
        pattern_id: 1,
        start_tick: 0,
        length_ticks: 384,
    });
    playlist.lanes[lane_idx].items.push(PlaylistItem::AudioClipInstance {
        clip_id: 101,
        start_tick: 384,
        length_ticks: 384,
        source_path: "/samples/vocal_drop.wav".to_string(),
    });
    playlist.lanes[lane_idx].items.push(PlaylistItem::AutomationClipInstance {
        clip_id: 201,
        start_tick: 0,
        length_ticks: 768,
    });

    assert_eq!(playlist.lanes[lane_idx].items.len(), 3);
}

#[test]
fn test_automation_clip_spline_tension() {
    let mut clip = AutomationClip::new(1, "Filter Cutoff", [0.0, 0.8, 0.9, 1.0], 768);
    clip.nodes.push(SplineNode {
        tick: 0,
        value: 0.0,
        tension: 0.5, // Exponential curve
    });
    clip.nodes.push(SplineNode {
        tick: 768,
        value: 1.0,
        tension: 0.0,
    });

    let mid_val = clip.evaluate(384);
    assert!(mid_val > 0.0 && mid_val < 1.0);
    assert_eq!(clip.evaluate(0), 0.0);
    assert_eq!(clip.evaluate(768), 1.0);
}

#[test]
fn test_patcher_modular_cables_and_macros() {
    let mut patcher = FlPatcher::new();
    let synth = patcher.add_node(PatcherNode::Generator {
        id: 1,
        name: "Sytrus".to_string(),
        plugin_id: 10,
    });
    let dist = patcher.add_node(PatcherNode::Effect {
        id: 2,
        name: "WaveShaper".to_string(),
        plugin_id: 20,
    });
    let out = patcher.add_node(PatcherNode::AudioOutput { id: 3 });

    patcher.add_cable(PatcherCable {
        from_node: synth,
        from_socket: 0,
        to_node: dist,
        to_socket: 0,
        color: [0.0, 1.0, 0.5, 1.0],
    });
    patcher.add_cable(PatcherCable {
        from_node: dist,
        from_socket: 0,
        to_node: out,
        to_socket: 0,
        color: [0.0, 1.0, 0.5, 1.0],
    });

    assert_eq!(patcher.cables.len(), 2);

    patcher.add_macro(PatcherMacroKnob {
        id: 1,
        name: "Aggression".to_string(),
        value: 0.75,
        target_node: dist,
        target_param: 1,
    });

    assert_eq!(patcher.macros[0].value, 0.75);
}

#[test]
fn test_gross_beat_time_and_volume_gating() {
    let mut gross_beat = GrossBeatEngine::new(44100.0);
    gross_beat.set_preset(GrossBeatPreset::HalfSpeed);

    let mut left = vec![0.5; 512];
    let mut right = vec![0.5; 512];

    gross_beat.process(&mut left, &mut right, 0);
    // Buffer processed without panic and altered
    assert_eq!(left.len(), 512);

    gross_beat.set_preset(GrossBeatPreset::TapeStop);
    gross_beat.process(&mut left, &mut right, 256);
    assert_eq!(right.len(), 512);
}

#[test]
fn test_fruity_soft_clipper_saturation() {
    let mut clipper = FruitySoftClipper::new(0.0, 0.8);

    let mut left = vec![2.5, -3.0, 0.1];
    let mut right = vec![2.5, -3.0, 0.1];

    clipper.process(&mut left, &mut right);

    // Hyperbolic tangent soft clipping saturates amplitudes smoothly without harsh overflow
    assert!(left[0] < 1.0 && left[0] > 0.0);
    assert!(left[1] > -1.0 && left[1] < 0.0);
}

#[test]
fn test_browser_smart_search_and_drop() {
    let mut browser = FlBrowser::new();
    browser.index_sample(SampleEntry {
        name: "808_Spinz_Clean.wav".to_string(),
        path: "/samples/trap/808_Spinz_Clean.wav".to_string(),
        category: "Kicks & 808s".to_string(),
        tags: vec!["808".to_string(), "trap".to_string(), "bass".to_string()],
        size_bytes: 1048576,
    });
    browser.index_sample(SampleEntry {
        name: "Snare_Lex_Luger.wav".to_string(),
        path: "/samples/trap/Snare_Lex_Luger.wav".to_string(),
        category: "Snares".to_string(),
        tags: vec!["snare".to_string(), "trap".to_string()],
        size_bytes: 524288,
    });

    let results = browser.search("808");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "808_Spinz_Clean.wav");

    // Drag-and-drop to channel rack
    let mut rack = ChannelRack::new(16);
    let ch = browser.drop_sample_to_rack(&results[0], &mut rack, 36);
    assert_eq!(ch, 0);
    assert_eq!(rack.channels[0].name, "808_Spinz_Clean.wav");
}

#[test]
fn test_visual_undo_history() {
    let mut history = FlHistoryManager::new(500);

    for i in 0..10 {
        history.record_action(
            &format!("Action {}", i),
            format!("Detail for action {}", i),
            format!("state_v{}", i),
        );
    }

    assert_eq!(history.actions.len(), 10);
    assert_eq!(history.current_index, 9);

    // Jump non-linearly to Action 3
    let state = history.jump_to_index(3);
    assert_eq!(state, Some("state_v3".to_string()));
    assert_eq!(history.current_index, 3);
}

#[test]
fn test_fl_window_manager_shortcuts() {
    let mut wm = FlWindowManager::new();

    assert!(wm.is_open(FlWindowKind::ChannelRack));
    assert!(wm.is_open(FlWindowKind::Playlist));

    // Toggle F7 (Piano Roll)
    wm.toggle(FlWindowKind::PianoRoll);
    assert!(wm.is_open(FlWindowKind::PianoRoll));

    // Handle key shortcut
    wm.handle_shortcut("F9");
    assert!(wm.is_open(FlWindowKind::Mixer));

    wm.handle_shortcut("Alt+F8");
    assert!(wm.is_open(FlWindowKind::Browser));
}
