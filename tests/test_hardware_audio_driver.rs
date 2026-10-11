use Crystal_wave::audio::AudioHardwareDriver;

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
