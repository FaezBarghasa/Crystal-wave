use Crystal_wave::dsp::SpectrumAnalyzer;
use std::f32::consts::PI;

#[test]
fn test_spectrum_analyzer_sine_wave_peak_detection() {
    let mut analyzer = SpectrumAnalyzer::new(0.0); // 0.0 smoothing for immediate response

    let mut samples = [0.0f32; 1024];
    let sample_rate = 44100.0;
    let target_freq = 1000.0; // 1 kHz sine wave

    for (i, s) in samples.iter_mut().enumerate() {
        *s = (2.0 * PI * target_freq * i as f32 / sample_rate).sin();
    }

    analyzer.process_frame(&samples);

    // Peak should be around bin index: round(1000 / 44100 * 1024) = 23
    let expected_bin = 23;
    let mut max_bin = 0;
    let mut max_db = -100.0f32;

    for (i, &db) in analyzer.magnitude_db.iter().enumerate() {
        if db > max_db {
            max_db = db;
            max_bin = i;
        }
    }

    assert!(
        (max_bin as isize - expected_bin as isize).abs() <= 1,
        "Max bin was {}, expected near {}",
        max_bin,
        expected_bin
    );
    assert!(max_db > -10.0, "Peak dB was {}, expected > -10.0", max_db);

    // Log bands should return 32 bands
    let log_bands = analyzer.get_log_bands(32);
    assert_eq!(log_bands.len(), 32);
}
