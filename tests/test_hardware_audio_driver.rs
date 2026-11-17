use Crystal_wave::audio::{AudioHardwareDriver, spawn_audio_worker};
use std::thread;
use std::time::Duration;

#[test]
fn test_audio_hardware_driver_initialization_and_metrics() {
    // Attempt initialization of default host audio driver
    match AudioHardwareDriver::new() {
        Ok(driver) => {
            assert!(!driver.host_name.is_empty());
            assert!(!driver.device_name.is_empty());
            assert!(driver.sample_rate >= 8000);
            assert!(driver.channels >= 1);
            assert!(!driver.is_active());
            assert_eq!(driver.underruns(), 0);
        }
        Err(err) => {
            // In headless CI/sandbox environments without physical ALSA/PulseAudio/JACK daemon,
            // driver initialization cleanly reports descriptive device error
            assert!(
                err.contains("No default audio output device")
                    || err.contains("device")
                    || err.contains("audio")
            );
        }
    }
}

#[test]
fn test_audio_worker_realtime_decoupling_and_telemetry() {
    let mut worker = spawn_audio_worker(44100.0, 256, None);
    assert!(!worker.is_playing());
    assert_eq!(worker.playhead_frame(), 0);
    assert_eq!(worker.underruns(), 0);

    // Transport start
    worker.play();
    assert!(worker.is_playing());

    // Allow worker OS thread to advance autonomously
    thread::sleep(Duration::from_millis(25));
    assert!(worker.playhead_frame() > 0);

    // Parameter updates via SPSC
    worker.set_tempo(145.0);
    worker.set_swing(0.65);

    // Pause transport
    worker.pause();
    assert!(!worker.is_playing());
    let paused_frame = worker.playhead_frame();
    thread::sleep(Duration::from_millis(15));
    assert_eq!(worker.playhead_frame(), paused_frame);

    // Seek exact frame
    worker.seek(88200);
    assert_eq!(worker.playhead_frame(), 88200);

    // Stop resets to origin
    worker.stop();
    assert!(!worker.is_playing());
    assert_eq!(worker.playhead_frame(), 0);
}

#[test]
fn test_audio_scheduler_fl_studio_odd_sixteenth_swing_math() {
    let mut scheduler = Crystal_wave::audio::TimelineAudioScheduler::new(44100.0, 120.0);
    assert_eq!(scheduler.swing_amount, 0.0);

    // With 0 swing, odd and even ticks are unchanged
    assert_eq!(scheduler.apply_swing_to_tick(0), 0);
    assert_eq!(scheduler.apply_swing_to_tick(240), 240);
    assert_eq!(scheduler.apply_swing_to_tick(480), 480);

    // Set 50% swing (0.50): delta = 0.5 * 120 = 60 ticks
    scheduler.set_swing(0.5);
    assert_eq!(scheduler.apply_swing_to_tick(0), 0); // step 0 (even): unchanged
    assert_eq!(scheduler.apply_swing_to_tick(240), 300); // step 1 (odd): 240 + 60 = 300
    assert_eq!(scheduler.apply_swing_to_tick(480), 480); // step 2 (even): unchanged
    assert_eq!(scheduler.apply_swing_to_tick(720), 780); // step 3 (odd): 720 + 60 = 780

    // Set 100% swing (1.0): delta = 1.0 * 120 = 120 ticks
    scheduler.set_swing(1.0);
    assert_eq!(scheduler.apply_swing_to_tick(240), 360); // step 1 (odd): 240 + 120 = 360
}
