use Crystal_wave::fl_studio::{
    automation_clip::AutomationClip,
    browser::{FlBrowser, SampleEntry},
    channel_rack::ChannelRack,
    gross_beat::{GrossBeatEngine, GrossBeatTimePreset, GrossBeatVolumePreset},
    history::FlHistoryManager,
    patcher::{FlPatcher, MacroMapping, PatcherNodeKind},
    piano_roll::{FlPianoRollNote, FlPianoRollTools, GhostNote},
    playlist::{FlPlaylist, PlaylistItem},
    soft_clipper::FruitySoftClipper,
    FlWindowManager,
};

#[test]
fn test_piano_roll_slide_notes_and_tools() {
    let mut notes = vec![
        FlPianoRollNote {
            id: 1,
            channel_id: 0,
            key: 60,
            start_tick: 0,
            length_ticks: 96,
            velocity: 0.8,
            pan: 0.0,
            is_slide: false,
            fine_pitch_cents: 0.0,
            color_group: 0,
        },
        FlPianoRollNote {
            id: 2,
            channel_id: 0,
            key: 64,
            start_tick: 96,
            length_ticks: 96,
            velocity: 0.8,
            pan: 0.0,
            is_slide: true,
            fine_pitch_cents: 0.0,
            color_group: 0,
        },
    ];

    assert!(notes[1].is_slide);

    // Strum tool
    FlPianoRollTools::quick_strum(&mut notes, 12, false);
    assert_eq!(notes[0].start_tick, 0);
    assert_eq!(notes[1].start_tick, 108);

    // Flip horizontal
    FlPianoRollTools::flip_horizontal(&mut notes, 108);
    assert_eq!(notes[1].start_tick, 108);

    // Flip vertical
    FlPianoRollTools::flip_vertical(&mut notes, 60);
    assert_eq!(notes[0].key, 60);

    // Chop
    let chopped = FlPianoRollTools::quick_chop(&notes[0], 4);
    assert_eq!(chopped.len(), 4);
    assert_eq!(chopped[0].length_ticks, 24);

    // Ghost notes representation
    let ghost = GhostNote {
        channel_id: 2,
        channel_name: "Bassline".to_string(),
        key: 67,
        start_tick: 0,
        length_ticks: 192,
        opacity: 0.35,
    };
    assert_eq!(ghost.channel_id, 2);
    assert_eq!(ghost.opacity, 0.35);
}

#[test]
fn test_channel_rack_and_graph_editor() {
    let mut rack = ChannelRack::new(16);
    let ch_idx = rack.add_channel("808 Bass");
    assert_eq!(ch_idx, 0);

    // Toggle steps
    rack.toggle_step(0, 0);
    rack.toggle_step(0, 4);
    rack.toggle_step(0, 8);
    rack.toggle_step(0, 12);

    assert!(rack.channels[0].steps[0].active);
    assert!(rack.channels[0].steps[4].active);
    assert!(!rack.channels[0].steps[1].active);

    // Graph editor per-step properties
    rack.channels[0].set_step_velocity(4, 0.95);
    rack.channels[0].set_step_pan(4, -0.2);
    rack.channels[0].steps[4].pitch_semitones = 12;

    assert_eq!(rack.channels[0].steps[4].pitch_semitones, 12);
    assert_eq!(rack.channels[0].steps[4].velocity, 0.95);
    assert_eq!(rack.channels[0].steps[4].pan, -0.2);

    // Export to MIDI note events
    let midi_events = rack.to_midi_events(0, 36);
    assert_eq!(midi_events.len(), 4);
    assert_eq!(midi_events[0].2, 36); // Base root key
    assert_eq!(midi_events[1].2, 48); // Root key + 12 semitones
}

#[test]
fn test_playlist_free_form_coexistence() {
    let mut playlist = FlPlaylist::new(8);
    let lane_idx = 0;

    let pat_id = playlist.create_pattern("Trap Drums");
    playlist.patterns[pat_id].add_note(FlPianoRollNote::new(1, 0, 36, 0, 96));

    // Place Pattern, Audio Clip, and Automation Clip all in the same lane
    assert!(playlist.stamp_item(lane_idx, PlaylistItem::PatternInstance {
        pattern_id: pat_id,
        start_tick: 0,
        length_ticks: 384,
    }));
    assert!(playlist.stamp_item(lane_idx, PlaylistItem::AudioClipInstance {
        sample_path: "/samples/vocal_drop.wav".to_string(),
        start_tick: 384,
        length_ticks: 384,
        gain_db: 0.0,
    }));
    assert!(playlist.stamp_item(lane_idx, PlaylistItem::AutomationClipInstance {
        clip_id: 201,
        start_tick: 0,
        length_ticks: 768,
    }));

    assert_eq!(playlist.lanes[lane_idx].items.len(), 3);
    assert_eq!(playlist.lanes[lane_idx].items[0].start_tick(), 0);
    assert_eq!(playlist.lanes[lane_idx].items[1].start_tick(), 384);
}

#[test]
fn test_automation_clip_spline_tension() {
    let mut clip = AutomationClip::new(1, "Filter Cutoff");
    clip.nodes.clear();
    clip.add_node(0, 0.0, 0.5);   // Convex curve
    clip.add_node(768, 1.0, 0.0);

    let mid_val = clip.evaluate(384);
    assert!(mid_val > 0.0 && mid_val < 1.0);
    assert_eq!(clip.evaluate(0), 0.0);
    assert_eq!(clip.evaluate(768), 1.0);
}

#[test]
fn test_patcher_modular_cables_and_macros() {
    let mut patcher = FlPatcher::new();
    let synth = patcher.add_node("Sytrus", PatcherNodeKind::GeneratorPlugin, 100.0, 100.0);
    let dist = patcher.add_node("WaveShaper", PatcherNodeKind::EffectPlugin, 250.0, 100.0);

    let connected = patcher.connect(synth, dist, 1.0);
    assert!(connected);
    // Patcher starts with 1 default cable (From FL -> To FL) + 1 newly added cable = 2 cables
    assert_eq!(patcher.cables.len(), 2);

    let macro_idx = patcher.add_macro("Aggression");
    patcher.macros[macro_idx].normalized_value = 0.75;
    patcher.macros[macro_idx].mappings.push(MacroMapping {
        target_node_id: dist,
        parameter_id: 1,
        min_value: 0.0,
        max_value: 1.0,
        invert: false,
    });

    assert_eq!(patcher.macros[0].normalized_value, 0.75);
}

#[test]
fn test_gross_beat_time_and_volume_gating() {
    let mut gross_beat = GrossBeatEngine::new(44100.0);
    gross_beat.time_preset = GrossBeatTimePreset::HalfSpeed;
    gross_beat.volume_preset = GrossBeatVolumePreset::SidechainPump;

    let mut left = vec![0.5; 512];
    let mut right = vec![0.5; 512];

    gross_beat.process(&mut left, &mut right);
    assert_eq!(left.len(), 512);

    gross_beat.time_preset = GrossBeatTimePreset::TapeStop;
    gross_beat.process(&mut left, &mut right);
    assert_eq!(right.len(), 512);
}

#[test]
fn test_fruity_soft_clipper_saturation() {
    let clipper = FruitySoftClipper::new();

    let mut left = vec![2.5, -3.0, 0.1];
    let mut right = vec![2.5, -3.0, 0.1];

    clipper.process(&mut left, &mut right);

    // Hyperbolic tangent soft clipping saturates amplitudes smoothly: input of 2.5 is compressed
    assert!(left[0] < 2.0 && left[0] > 0.0);
    assert!(left[1] > -2.0 && left[1] < 0.0);
}

#[test]
fn test_browser_smart_search_and_drop() {
    let mut browser = FlBrowser::new();
    browser.samples.push(SampleEntry {
        id: 101,
        name: "808_Spinz_Clean.wav".to_string(),
        category: "808".to_string(),
        file_path: "/samples/trap/808_Spinz_Clean.wav".to_string(),
        duration_seconds: 2.1,
    });
    browser.samples.push(SampleEntry {
        id: 102,
        name: "Snare_Lex_Luger.wav".to_string(),
        category: "Snares".to_string(),
        file_path: "/samples/trap/Snare_Lex_Luger.wav".to_string(),
        duration_seconds: 0.6,
    });

    let results = browser.search("808");
    assert_eq!(results.len(), 3); // Includes 2 default 808s + 1 newly added

    // Drag-and-drop to channel rack
    let mut rack = ChannelRack::new(16);
    let ch = browser.drop_to_channel_rack(&mut rack, 101);
    assert_eq!(ch, Some(0));
    assert_eq!(rack.channels[0].name, "808_Spinz_Clean.wav");
    assert_eq!(rack.channels[0].sample_path, Some("/samples/trap/808_Spinz_Clean.wav".to_string()));
}

#[test]
fn test_visual_undo_history() {
    let mut history: FlHistoryManager<String> = FlHistoryManager::new();

    for i in 0..10 {
        history.record_action(
            format!("Action {}", i),
            format!("state_v{}", i),
        );
    }

    assert_eq!(history.list_history().len(), 10);
    assert_eq!(history.current_action().unwrap().description, "Action 9");

    // Jump non-linearly to Action 3
    let state = history.jump_to(3);
    assert_eq!(state, Some(&"state_v3".to_string()));
    assert_eq!(history.current_action().unwrap().description, "Action 3");
}

#[test]
fn test_fl_window_manager_shortcuts() {
    let mut wm = FlWindowManager::new();

    assert!(wm.channel_rack_visible);
    assert!(wm.playlist_visible);
    assert!(!wm.piano_roll_visible);

    // Toggle F7 (Piano Roll)
    wm.toggle_piano_roll();
    assert!(wm.piano_roll_visible);

    // Toggle F9 (Mixer)
    wm.toggle_mixer();
    assert!(wm.mixer_visible);

    // Toggle Alt+F8 (Browser)
    wm.toggle_browser();
    assert!(!wm.browser_visible);
}
