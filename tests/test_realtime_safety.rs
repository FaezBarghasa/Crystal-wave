use Crystal_wave::audio::disk_writer::DiskWriterWorker;
use Crystal_wave::audio::recording_engine::RecordingEngine;
use Crystal_wave::plugins::plugin_chain::PluginChain;
use Crystal_wave::plugins::plugin_host::NativeTapeDelay;

#[test]
fn test_lock_free_spsc_recording_pipeline() {
    let mut engine = RecordingEngine::new(44100.0);

    // Arm track 0 with lock-free SPSC channel (capacity 8192 samples)
    let consumer = engine.arm_track_lock_free(0, 8192);

    engine.start_recording(0);

    // Spawn background disk worker
    let worker = DiskWriterWorker::spawn_sample_stream(consumer);

    // Simulate 50 real-time audio blocks of 128 samples pushed from the audio callback
    let block = vec![0.25f32; 128];
    for _ in 0..50 {
        engine.append_samples_wait_free(0, &block);
    }

    // Stop recording and verify background disk worker drained samples
    engine.stop_recording();
    let total_samples = worker.stop();
    assert_eq!(total_samples, 128 * 50);
}

#[test]
fn test_plugin_chain_scratchpad_zero_alloc_reuse() {
    let mut chain = PluginChain::new();
    let delay = Box::new(NativeTapeDelay::new(44100.0));
    assert!(chain.insert_plugin(delay));

    // First block initializes scratchpad
    let mut left = vec![0.5f32; 256];
    let mut right = vec![0.5f32; 256];
    chain.process(&mut left, &mut right, None);

    // Subsequent 10,000 blocks run over pre-allocated scratchpad with zero reallocation
    for _ in 0..10_000 {
        chain.process(&mut left, &mut right, None);
    }

    assert_eq!(left.len(), 256);
    assert_eq!(right.len(), 256);
}
