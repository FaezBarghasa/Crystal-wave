use std::f32::consts::PI;
use Crystal_wave::dsp::loudness_ebu_r128::EbuR128LoudnessMeter;

#[test]
fn test_ebu_r128_loudness_metering_and_true_peak() {
    let sample_rate = 48000.0f32;
    let mut meter = EbuR128LoudnessMeter::new(sample_rate, 2);

    assert_eq!(meter.sample_rate(), 48000.0);

    // Initial state
    assert_eq!(meter.momentary_lufs(), -70.0);
    assert_eq!(meter.short_term_lufs(), -70.0);
    assert_eq!(meter.integrated_lufs(), -70.0);
    assert_eq!(meter.max_true_peak_dbtp(), -120.0);

    // 1. Generate 1.0 second of 1 kHz stereo sine wave at -23.0 dBFS (standard EBU R128 target)
    let target_db = -23.0f32;
    let amplitude = 10.0f32.powf(target_db / 20.0); // ~0.07079
    let num_samples = 48000;
    let mut left = vec![0.0f32; num_samples];
    let mut right = vec![0.0f32; num_samples];

    for i in 0..num_samples {
        let val = amplitude * (2.0 * PI * 1000.0 * (i as f32) / sample_rate).sin();
        left[i] = val;
        right[i] = val;
    }

    meter.process_channels(&[&left, &right]);

    // Check Momentary and Short-term loudness
    // With K-weighting shelf boost near 1 kHz, -23 dBFS sine typically reads ~ -20 to -23 LUFS
    let m_lufs = meter.momentary_lufs();
    let s_lufs = meter.short_term_lufs();
    let i_lufs = meter.integrated_lufs();

    assert!(m_lufs > -25.0 && m_lufs < -18.0, "Momentary LUFS was {}", m_lufs);
    assert!(s_lufs > -25.0 && s_lufs < -18.0, "Short-term LUFS was {}", s_lufs);
    assert!(i_lufs > -25.0 && i_lufs < -18.0, "Integrated LUFS was {}", i_lufs);

    // Peak level should be close to -23.0 dBTP
    let peak = meter.max_true_peak_dbtp();
    assert!((peak - (-23.0)).abs() < 1.0, "Peak dBTP was {}", peak);

    // 2. Feed silence (0.5s) to verify absolute gating (-70 LKFS threshold)
    // The integrated loudness must remain around -20 to -23 LUFS because silence is gated out
    let silence = vec![0.0f32; 24000];
    meter.process_channels(&[&silence, &silence]);

    let i_lufs_gated = meter.integrated_lufs();
    assert!((i_lufs_gated - i_lufs).abs() < 0.5, "Integrated loudness changed from {} to {}", i_lufs, i_lufs_gated);

    // 3. Reset
    meter.reset();
    assert_eq!(meter.momentary_lufs(), -70.0);
    assert_eq!(meter.integrated_lufs(), -70.0);
}
