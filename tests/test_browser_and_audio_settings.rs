use Crystal_wave::fl_studio::browser::{BrowserTab, FlBrowser};
use Crystal_wave::fl_studio::channel_rack::ChannelRack;
use Crystal_wave::plugins::plugin_descriptor::PluginCategory;
use Crystal_wave::timeline::region::RegionKind;
use Crystal_wave::timeline::track::{Track, TrackType};
use Crystal_wave::ui::app::CrystalWaveApp;
use Crystal_wave::ui::message::{AudioBackend, UiMessage};

#[test]
fn test_browser_categories_tabs_and_smart_search() {
    let mut browser = FlBrowser::new();

    // Verify default All tab contains all samples
    assert_eq!(browser.active_tab, BrowserTab::All);
    let all_samples = browser.filtered_samples();
    assert_eq!(all_samples.len(), browser.samples.len());

    // Switch to Drums tab
    browser.active_tab = BrowserTab::Drums;
    let drum_samples = browser.filtered_samples();
    assert!(!drum_samples.is_empty());
    for s in &drum_samples {
        assert!(s.category == "Kicks" || s.category == "Snares" || s.category == "HiHats");
    }

    // Switch to Bass tab (808)
    browser.active_tab = BrowserTab::Bass;
    let bass_samples = browser.filtered_samples();
    assert!(!bass_samples.is_empty());
    for s in &bass_samples {
        assert_eq!(s.category, "808");
    }

    // Switch to FX tab
    browser.active_tab = BrowserTab::Fx;
    let fx_samples = browser.filtered_samples();
    assert!(!fx_samples.is_empty());
    for s in &fx_samples {
        assert_eq!(s.category, "FX");
    }

    // Test smart search query
    browser.active_tab = BrowserTab::All;
    browser.search_query = "spinz".into();
    let query_res = browser.filtered_samples();
    assert_eq!(query_res.len(), 1);
    assert_eq!(query_res[0].name, "Spinz 808.wav");

    // Test preset filtering
    browser.search_query = "sytrus".into();
    let preset_res = browser.filtered_presets();
    assert_eq!(preset_res.len(), 1);
    assert_eq!(preset_res[0].name, "Bell Chime Pluck");

    // Test plugin filtering
    let synths = browser.filtered_plugins("", Some(PluginCategory::Synth));
    assert!(synths.iter().any(|p| p.name == "3x OSC"));
    assert!(synths.iter().any(|p| p.name == "Sytrus"));
}

#[test]
fn test_browser_audition_and_drag_drop_targets() {
    let mut browser = FlBrowser::new();
    let mut rack = ChannelRack::new(16);
    let mut tracks = vec![
        Track::new(0, "Track 1", TrackType::Audio),
        Track::new(1, "Track 2", TrackType::Audio),
    ];

    // Audition preview
    assert_eq!(browser.previewing_sample_id, None);
    browser.start_preview(1);
    assert_eq!(browser.previewing_sample_id, Some(1));
    browser.stop_preview();
    assert_eq!(browser.previewing_sample_id, None);

    // Drop to Channel Rack
    let chan_id = browser.drop_to_channel_rack(&mut rack, 1);
    assert!(chan_id.is_some());
    let id = chan_id.unwrap();
    assert_eq!(rack.channels[id].name, "Spinz 808.wav");
    assert_eq!(
        rack.channels[id].sample_path.as_deref(),
        Some("/samples/808/Spinz 808.wav")
    );

    // Drop to Playlist Track
    let reg_id = browser.drop_to_playlist(&mut tracks, 2, 1, 960, 1920);
    assert!(reg_id.is_some());
    assert_eq!(tracks[1].regions.len(), 1);
    let region = &tracks[1].regions[0];
    assert_eq!(region.name, "Rack Kick.wav");
    assert_eq!(region.start_tick, 960);
    assert_eq!(region.length_ticks, 1920);
    assert!(matches!(region.kind, RegionKind::Audio { .. }));
}

#[test]
fn test_visual_plugin_picker_filtering_and_assignment() {
    let mut app = CrystalWaveApp::new(44100.0);

    // 1. Open Plugin Picker for insert slot
    app.update(UiMessage::OpenPluginPickerForSlot {
        track_id: 1,
        slot_id: 2,
    });
    assert!(app.fl_windows.plugin_picker_visible);
    assert_eq!(app.plugin_picker_target_slot, Some((1, 2)));

    // 2. Filter by category Dynamics
    app.update(UiMessage::PluginPickerSelectCategory(Some(
        PluginCategory::Dynamics,
    )));
    assert_eq!(app.plugin_picker_category, Some(PluginCategory::Dynamics));

    // 3. Load Fruity Soft Clipper (id 1004) into target slot
    app.update(UiMessage::PluginPickerLoadPlugin {
        plugin_id: 1004,
        is_generator: false,
    });
    assert!(!app.fl_windows.plugin_picker_visible);

    // Verify track 1 slot 2 now has Fruity Soft Clipper
    let strip = app.mixer.channels.get(1).expect("channel 1 must exist");
    let slot = &strip.slots[2];
    assert!(!slot.bypassed);
    assert_eq!(slot.plugin_id, Some(1004));
    assert_eq!(slot.name, "Fruity Soft Clipper");

    // 4. Load generator (Synth) into Channel Rack
    let prev_channels = app.channel_rack.channels.len();
    app.update(UiMessage::PluginPickerLoadPlugin {
        plugin_id: 2001, // 3x OSC
        is_generator: true,
    });
    assert_eq!(app.channel_rack.channels.len(), prev_channels + 1);
    let last_chan = app.channel_rack.channels.last().unwrap();
    assert_eq!(last_chan.name, "3x OSC");
}

#[test]
fn test_audio_settings_backend_buffer_and_sample_rate() {
    let mut app = CrystalWaveApp::new(44100.0);

    // Toggle settings modal
    assert!(!app.audio_settings_open);
    app.update(UiMessage::ToggleAudioSettings);
    assert!(app.audio_settings_open);

    // Change Backend to ALSA
    app.update(UiMessage::SetAudioBackend(AudioBackend::Alsa));
    assert_eq!(app.audio_backend, AudioBackend::Alsa);

    // Change Buffer size to 512
    app.update(UiMessage::SetAudioBufferSize(512));
    assert_eq!(app.audio_buffer_size, 512);

    // Change Sample Rate to 96000
    app.update(UiMessage::SetAudioSampleRate(96000));
    assert_eq!(app.audio_sample_rate_selection, 96000);
    assert_eq!(app.audio_scheduler.sample_rate, 96000.0);
    assert_eq!(app.mixer.sample_rate, 96000.0);

    // Compute expected latency: (512 / 96000) * 1000 = 5.333 ms
    let latency_ms =
        (app.audio_buffer_size as f32 / app.audio_sample_rate_selection as f32) * 1000.0;
    assert!((latency_ms - 5.3333).abs() < 0.01);

    // Reset XRUNs
    if let Some(ref mut worker) = app.audio_worker {
        worker
            .state
            .underruns
            .store(5, std::sync::atomic::Ordering::Relaxed);
    }
    app.update(UiMessage::ResetAudioXruns);
    if let Some(ref worker) = app.audio_worker {
        assert_eq!(
            worker
                .state
                .underruns
                .load(std::sync::atomic::Ordering::Relaxed),
            0
        );
    }
}

#[test]
fn test_ui_view_with_plugin_picker_and_audio_settings_open() {
    let mut app = CrystalWaveApp::new(44100.0);

    // Verify base view
    let _ = app.view();

    // Open Plugin Picker and render
    app.fl_windows.plugin_picker_visible = true;
    app.plugin_picker_search = "clip".into();
    let _ = app.view();

    // Open Audio Settings modal and render
    app.audio_settings_open = true;
    let _ = app.view();

    // Open Browser with Presets tab and render
    app.fl_windows.browser_visible = true;
    app.browser.active_tab = BrowserTab::Presets;
    let _ = app.view();

    // Switch Browser to Plugins tab and render
    app.browser.active_tab = BrowserTab::Plugins;
    let _ = app.view();

    // Switch Browser to Drums tab and render
    app.browser.active_tab = BrowserTab::Drums;
    app.browser.start_preview(2);
    let _ = app.view();
}
