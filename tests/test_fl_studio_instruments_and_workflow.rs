//! Integration tests for FL Studio dedicated instruments, routing, and workflow
//!
//! Validates:
//! 1. Edison Audio Editor operations (normalize, fade, reverse, clipboard)
//! 2. FPC 16-pad MPC drum sampler (pad triggers, kit loading, volume)
//! 3. Slicex transient slicer (markers, triggering, reset)
//! 4. Fruity Wrapper ADSR envelopes and channel routing
//! 5. CrystalWaveApp FL view routing and F-key shortcuts
//! 6. Mixer send routing matrix (cable send logic)
//! 7. Tap tempo and tempo nudging

use Crystal_wave::fl_studio::edison::EdisonEditor;
use Crystal_wave::fl_studio::fpc::FpcEngine;
use Crystal_wave::fl_studio::slicex::SlicexEngine;
use Crystal_wave::fl_studio::wrapper::{
    AdsrEnvelope, FruityWrapperState, PolyphonyMode, WrapperTab,
};
use Crystal_wave::ui::app::CrystalWaveApp;
use Crystal_wave::ui::layout::FlActiveView;
use Crystal_wave::ui::message::UiMessage;

#[test]
fn test_edison_full_workflow() {
    let mut edison = EdisonEditor::new(48000.0);
    assert_eq!(edison.sample_rate, 48000.0);

    // Initial dummy buffer is populated
    assert!(!edison.is_empty());
    assert!(edison.len() > 1000);

    // Select range and normalize
    edison.set_selection(100, 500);
    assert_eq!(edison.selection_start, 100);
    assert_eq!(edison.selection_end, 500);

    edison.normalize();
    let (s, e) = edison.active_range();
    let mut peak: f32 = 0.0;
    for i in s..e {
        peak = peak
            .max(edison.samples_l[i].abs())
            .max(edison.samples_r[i].abs());
    }
    assert!((peak - 1.0).abs() < 1e-4);

    // Fade in and out
    edison.fade_in();
    assert_eq!(edison.samples_l[s], 0.0);

    edison.fade_out();
    assert_eq!(edison.samples_l[e - 1], 0.0);

    // Clipboard copy/paste
    edison.copy();
    assert_eq!(edison.clipboard_l.len(), 400);

    let original_len = edison.len();
    edison.cursor_pos = 0;
    edison.paste();
    assert_eq!(edison.len(), original_len + 400);
}

#[test]
fn test_fpc_full_drum_machine() {
    let mut fpc = FpcEngine::new();
    assert_eq!(fpc.pads.len(), 16);

    // Verify 4x4 pad matrix MIDI notes (C2..D#3)
    assert_eq!(fpc.pads[0].note_number, 36); // C2 Kick 1
    assert_eq!(fpc.pads[0].name, "Kick 1");
    assert_eq!(fpc.pads[1].name, "Snare 1");
    assert_eq!(fpc.pads[2].name, "Hi-Hat Closed");

    // Pad trigger verification
    fpc.trigger_pad(2);
    assert!(fpc.pads[2].is_triggered);
    assert!(!fpc.pads[0].is_triggered);

    fpc.reset_triggers();
    assert!(!fpc.pads[2].is_triggered);

    // Kit changing
    fpc.load_kit("808 Pierre Kit");
    assert_eq!(fpc.active_kit_name, "808 Pierre Kit");
}

#[test]
fn test_slicex_transient_detection_and_triggers() {
    let mut slicex = SlicexEngine::new(44100.0);
    assert_eq!(slicex.markers.len(), 8);
    assert_eq!(slicex.sample_name, "Amen Break 174BPM.wav");

    // Trigger slice 3
    slicex.trigger_slice(3);
    assert_eq!(slicex.active_slice_id, Some(3));
    assert!(slicex.markers[3].is_active);
    assert!(!slicex.markers[0].is_active);

    slicex.reset_triggers();
    assert_eq!(slicex.active_slice_id, None);
    assert!(!slicex.markers[3].is_active);
}

#[test]
fn test_fruity_wrapper_channel_state() {
    let mut wrapper = FruityWrapperState::new(2);
    assert_eq!(wrapper.channel_id, 2);
    assert_eq!(wrapper.target_mixer_track, 3);
    assert_eq!(wrapper.polyphony_mode, PolyphonyMode::Poly);

    wrapper.volume_env = AdsrEnvelope {
        attack_ms: 15.0,
        decay_ms: 150.0,
        sustain_level: 0.65,
        release_ms: 450.0,
        tension: 0.2,
        enabled: true,
    };
    assert_eq!(wrapper.volume_env.attack_ms, 15.0);
    assert_eq!(wrapper.volume_env.decay_ms, 150.0);
    assert_eq!(wrapper.volume_env.sustain_level, 0.65);
    assert_eq!(wrapper.volume_env.release_ms, 450.0);

    wrapper.active_tab = WrapperTab::Envelope;
    assert_eq!(wrapper.active_tab, WrapperTab::Envelope);
}

#[test]
fn test_fl_studio_view_navigation_and_f_keys() {
    let mut app = CrystalWaveApp::new(48000.0);

    // Default view is Playlist
    assert_eq!(app.layout.active_view, FlActiveView::Playlist);

    // F6 -> Channel Rack
    app.update(UiMessage::ToggleFlChannelRack);
    assert_eq!(app.layout.active_view, FlActiveView::ChannelRack);

    // F7 -> Piano Roll
    app.update(UiMessage::ToggleFlPianoRoll);
    assert_eq!(app.layout.active_view, FlActiveView::PianoRoll);

    // F9 -> Mixer
    app.update(UiMessage::ToggleFlMixer);
    assert_eq!(app.layout.active_view, FlActiveView::Mixer);

    // F5 -> Playlist
    app.update(UiMessage::ToggleFlPlaylist);
    assert_eq!(app.layout.active_view, FlActiveView::Playlist);

    // Edison Toggle
    app.update(UiMessage::ToggleFlEdison);
    assert_eq!(app.layout.active_view, FlActiveView::Edison);

    // FPC Toggle
    app.update(UiMessage::ToggleFlFpc);
    assert_eq!(app.layout.active_view, FlActiveView::Fpc);

    // Slicex Toggle
    app.update(UiMessage::ToggleFlSlicex);
    assert_eq!(app.layout.active_view, FlActiveView::Slicex);

    // F10 Audio Settings Toggle
    assert!(!app.audio_settings_open);
    app.update(UiMessage::ToggleAudioSettings);
    assert!(app.audio_settings_open);
    assert!(app.fl_windows.audio_settings_visible);

    app.update(UiMessage::ToggleAudioSettings);
    assert!(!app.audio_settings_open);
    assert!(!app.fl_windows.audio_settings_visible);
}

#[test]
fn test_mixer_routing_sends_matrix() {
    let mut app = CrystalWaveApp::new(48000.0);
    assert_eq!(app.mixer_sends.len(), 128);

    // By default, track 1 sends to Master (track 0) with level 1.0
    assert_eq!(app.mixer_sends[1][0], 1.0);

    // Toggle track 1 send to Master off
    app.update(UiMessage::MixerToggleSend {
        from_track: 1,
        to_track: 0,
    });
    assert_eq!(app.mixer_sends[1][0], 0.0);

    // Toggle track 1 send to Master on
    app.update(UiMessage::MixerToggleSend {
        from_track: 1,
        to_track: 0,
    });
    assert_eq!(app.mixer_sends[1][0], 0.8);

    // Set send level
    app.update(UiMessage::MixerSetSendLevel {
        from_track: 1,
        to_track: 0,
        level: 0.5,
    });
    assert_eq!(app.mixer_sends[1][0], 0.5);
}

#[test]
fn test_tempo_nudge_and_tap() {
    let mut app = CrystalWaveApp::new(48000.0);
    let initial_bpm = app.workspace.lcd_diagnostic.project_tempo_bpm;

    // Nudge tempo +1.0 BPM
    app.update(UiMessage::NudgeTempo(1.0));
    assert!((app.workspace.lcd_diagnostic.project_tempo_bpm - (initial_bpm + 1.0)).abs() < 1e-4);

    // Nudge tempo -0.5 BPM
    app.update(UiMessage::NudgeTempo(-0.5));
    assert!((app.workspace.lcd_diagnostic.project_tempo_bpm - (initial_bpm + 0.5)).abs() < 1e-4);

    // Tap tempo records timestamps
    app.update(UiMessage::TapTempo);
    assert_eq!(app.tap_tempo_times.len(), 1);
}
