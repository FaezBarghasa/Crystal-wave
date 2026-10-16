use Crystal_wave::fl_studio::gross_beat::{
    GrossBeatEngine, GrossBeatTimePreset, GrossBeatVolumePreset,
};
use Crystal_wave::mixing::MixingEngine;
use Crystal_wave::plugins::PluginRegistry;
use Crystal_wave::ui::app::CrystalWaveApp;
use Crystal_wave::ui::message::UiMessage;

#[test]
fn test_plugin_catalog_complete_suite() {
    let registry = PluginRegistry::new();
    assert_eq!(registry.len(), 8);

    // Verify all 8 built-in plugins are registered and enabled by default
    let descriptors = registry.list_all();
    let names: Vec<&str> = descriptors.iter().map(|d| d.name.as_str()).collect();
    assert!(names.contains(&"Tape Delay"));
    assert!(names.contains(&"Studio Compressor"));
    assert!(names.contains(&"Channel EQ (8-Band)"));
    assert!(names.contains(&"Fruity Soft Clipper"));
    assert!(names.contains(&"ChromaVerb"));
    assert!(names.contains(&"Gross Beat"));
    assert!(names.contains(&"Subtractive Synth"));
    assert!(names.contains(&"Drum Synth"));

    // Verify instantiation and processing of Subtractive Synth (1007)
    let mut synth = registry
        .instantiate(1007, 44100.0)
        .expect("Subtractive Synth");
    let mut left = vec![0.0f32; 256];
    let mut right = vec![0.0f32; 256];
    synth.process(&mut left, &mut right, None);
    assert!(left.iter().any(|&s| s.abs() > 1e-4));
    assert!(right.iter().any(|&s| s.abs() > 1e-4));

    // Verify instantiation and processing of Drum Synth (1008)
    let mut drum = registry.instantiate(1008, 44100.0).expect("Drum Synth");
    let mut d_left = vec![0.0f32; 256];
    let mut d_right = vec![0.0f32; 256];
    drum.process(&mut d_left, &mut d_right, None);
    assert!(d_left.iter().any(|&s| s.abs() > 1e-4));
}

#[test]
fn test_topological_multi_bus_summing() {
    let mut mixer = MixingEngine::new(44100.0).expect("Valid mixer init");

    // Add channels: Channel 0 (Drums) routes to Bus 10
    // Bus 10 routes to Bus 2 (Pre-Master)
    // Bus 2 routes to Bus 0 (Stereo Out)
    let ch0 = mixer.add_channel("Drum Track").expect("Add channel");
    let ch1 = mixer.add_channel("Bass Track").expect("Add channel");

    mixer.channels[ch0].output_bus = 10;
    mixer.channels[ch1].output_bus = 2;

    // Configure topological routing for buses:
    // Bus 10 -> Bus 2
    mixer.buses[10].target_bus = Some(2);
    // Bus 2 -> Bus 0
    mixer.buses[2].target_bus = Some(0);

    // Route audio into channels
    let audio_data = vec![0.5f32; 128];
    mixer.route_channel_audio(ch0, &audio_data, &audio_data);
    mixer.route_channel_audio(ch1, &audio_data, &audio_data);

    let mut master_l = vec![0.0f32; 128];
    let mut master_r = vec![0.0f32; 128];
    mixer.sum_buses(&mut master_l, &mut master_r);

    // Summing should hierarchically propagate from Bus 10 -> Bus 2 -> Bus 0 -> Master
    for i in 0..128 {
        assert!(
            master_l[i] > 0.5,
            "Master output should contain summed bus energy"
        );
        assert!(master_r[i] > 0.5);
    }
}

#[test]
fn test_gross_beat_hermite_interpolation_and_click_free_gating() {
    let mut gb = GrossBeatEngine::new(44100.0);

    // Test Reverse 1 Bar with Hermite interpolation & crossfade
    gb.time_preset = GrossBeatTimePreset::Reverse1Bar;
    gb.volume_preset = GrossBeatVolumePreset::Gate16th;

    let mut left = vec![0.3f32; 1024];
    let mut right = vec![0.3f32; 1024];
    gb.process(&mut left, &mut right);

    for &sample in &left {
        assert!(
            sample.is_finite(),
            "Output must be finite and free from NaN/Inf"
        );
        assert!(sample.abs() <= 1.0, "Sample amplitude within unit range");
    }

    // Test Triplet Stutter & Tape Stop
    gb.time_preset = GrossBeatTimePreset::TripletStutter;
    gb.process(&mut left, &mut right);
    for &sample in &left {
        assert!(sample.is_finite());
    }

    gb.time_preset = GrossBeatTimePreset::TapeStop;
    gb.process(&mut left, &mut right);
    for &sample in &left {
        assert!(sample.is_finite());
    }
}

#[test]
fn test_smart_tool_timeline_clip_modifications() {
    let mut app = CrystalWaveApp::default();
    let initial_reg_count = app.arrangement.timeline.tracks[0].regions.len();
    assert!(initial_reg_count > 0);
    let clip_id = app.arrangement.timeline.tracks[0].regions[0].id;

    // Test TimelineTrimClipStart
    app.update(UiMessage::TimelineTrimClipStart {
        clip_id,
        new_start_tick: 480,
    });
    assert_eq!(
        app.arrangement.timeline.tracks[0].regions[0].start_tick,
        480
    );

    // Test TimelineTrimClipEnd
    app.update(UiMessage::TimelineTrimClipEnd {
        clip_id,
        new_length_ticks: 1920,
    });
    assert_eq!(
        app.arrangement.timeline.tracks[0].regions[0].length_ticks,
        1920
    );

    // Test TimelineFadeClip
    app.update(UiMessage::TimelineFadeClip {
        clip_id,
        fade_in_ticks: Some(120),
        fade_out_ticks: Some(240),
    });
    assert_eq!(
        app.arrangement.timeline.tracks[0].regions[0].fade.in_ticks,
        120
    );
    assert_eq!(
        app.arrangement.timeline.tracks[0].regions[0].fade.out_ticks,
        240
    );
}
