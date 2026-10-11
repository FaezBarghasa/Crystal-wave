use Crystal_wave::audio::*;
use Crystal_wave::audio_edit::*;
use std::sync::{Arc, Mutex};

#[test]
fn test_recording_engine_and_circular_buffer() {
    let mut engine = RecordingEngine::new(44100.0);
    engine.arm_track(0);
    assert!(engine.is_armed(0));

    // Circular buffer should hold incoming samples even when stopped
    let test_samples = vec![0.5f32; 100];
    engine.append_samples(0, &test_samples);
    let captured = engine.circular_buffers[0].read_latest(50);
    assert_eq!(captured.len(), 50);
    assert_eq!(captured[0], 0.5);

    // Track buffer should be empty since recording hasn't started
    assert!(engine.track_buffers[0].lock().unwrap().is_empty());

    // Start recording
    engine.start_recording(0);
    engine.append_samples(0, &test_samples);
    assert_eq!(engine.track_buffers[0].lock().unwrap().len(), 100);

    // Commit take
    let take = engine
        .commit_take(0, "Vocal Take 1")
        .expect("Take committed");
    assert_eq!(take.name, "Vocal Take 1");
    assert_eq!(take.len(), 100);
}

#[test]
fn test_auto_punch_gating() {
    let mut engine = RecordingEngine::new(44100.0);
    engine.arm_track(1);
    engine.set_auto_punch(100, 300, true);
    engine.start_recording(0);

    // Append 0..100 (before punch) -> should not record
    engine.append_samples(1, &vec![1.0; 100]);
    assert_eq!(engine.track_buffers[1].lock().unwrap().len(), 0);

    // Append 100..200 (inside punch) -> should record
    engine.append_samples(1, &vec![1.0; 100]);
    assert_eq!(engine.track_buffers[1].lock().unwrap().len(), 100);
}

#[test]
fn test_quick_swipe_comping_and_export() {
    let mut comp = CompingTool::with_default_crossfade(44100.0);
    let take1 = Arc::new(Mutex::new(Take::new(1, "Take 1", vec![0.8; 1000])));
    let take2 = Arc::new(Mutex::new(Take::new(2, "Take 2", vec![0.2; 1000])));
    comp.add_take(take1);
    comp.add_take(take2);

    comp.add_selection(1, 0, 500);
    comp.add_selection(2, 500, 1000);

    let flattened = comp.flatten_and_merge(1000);
    assert_eq!(flattened.len(), 1000);
    assert!((flattened[100] - 0.8).abs() < 1e-3);
    assert!((flattened[900] - 0.2).abs() < 1e-3);

    let comp_master = comp.export_comp_to_track(1000);
    assert_eq!(comp_master.name, "Comp Master");
    assert_eq!(comp_master.len(), 1000);
}

#[test]
fn test_audio_file_editor_strip_silence_and_dc() {
    // Buffer with small DC offset of 0.05 during silence, and loud alternating tone
    let mut buf = vec![0.05; 4000]; // DC offset 0.05 in silence
    // Section 1: 500..1500 (alternating loud square tone centered around 0.05)
    for (i, sample) in buf[500..1500].iter_mut().enumerate() {
        *sample = if (i + 500) % 2 == 0 { 0.85 } else { -0.75 };
    }
    // Section 2: 2500..3500 (alternating loud square tone centered around 0.05)
    for (i, sample) in buf[2500..3500].iter_mut().enumerate() {
        *sample = if (i + 2500) % 2 == 0 { 0.85 } else { -0.75 };
    }

    let mut editor = AudioFileEditor::new(buf, 1000.0);
    editor.remove_dc_offset();

    let cfg = StripSilenceConfig {
        threshold_db: -12.0, // ~0.25
        min_section_length_ms: 100.0,
        pre_attack_ms: 10.0,
        post_release_ms: 10.0,
    };
    let slices = editor.strip_silence(&cfg);
    assert_eq!(slices.len(), 2);
    assert!(slices[0].0 <= 500 && slices[0].1 >= 1500);
    assert!(slices[1].0 <= 2500 && slices[1].1 >= 3500);
}

#[test]
fn test_pencil_edit_repair_pop_click() {
    let mut buf = vec![0.0f32; 100];
    // Baseline signal at 0.5
    buf.fill(0.5);
    // Discontinuous loud spike / click at index 48..52
    buf[48..52].fill(1.0);

    PencilEdit::repair_pop_click(&mut buf, 48, 52);
    // Spike at 50 should be smoothed back down near 0.5
    assert!((buf[50] - 0.5).abs() < 0.1);
}
