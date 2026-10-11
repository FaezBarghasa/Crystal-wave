use Crystal_wave::dsp::equalizer::{ChannelEq, FilterType};
use Crystal_wave::fl_studio::soft_clipper::FruitySoftClipper;
use Crystal_wave::mixing::channel_strip::ChannelStrip;

#[test]
fn test_channel_strip_insert_plugins_execution() {
    let mut strip = ChannelStrip::new(0, "Vocal Track");

    // Initially no plugins
    assert_eq!(strip.plugins.slots.len(), 0);

    // 1. Insert 8-Band Channel EQ with high boost at 1kHz
    let mut eq = ChannelEq::new(44100.0).expect("EQ init failed");
    eq.bands[3].filter_type = FilterType::Peak;
    eq.bands[3].freq_hz = 1000.0;
    eq.bands[3].gain_db = 12.0; // +12 dB boost
    eq.bands[3].update(44100.0);

    let inserted_eq = strip.plugins.insert_plugin(Box::new(eq));
    assert!(inserted_eq);
    assert_eq!(strip.plugins.slots.len(), 1);

    // 2. Insert Fruity Soft Clipper
    let mut clipper = FruitySoftClipper::new();
    clipper.threshold_db = -6.0;
    clipper.post_gain_db = 0.0;
    let inserted_clipper = strip.plugins.insert_plugin(Box::new(clipper));
    assert!(inserted_clipper);
    assert_eq!(strip.plugins.slots.len(), 2);

    // 3. Process signal through channel strip
    let mut left = vec![1.5f32; 128]; // Signal exceeding soft clipper threshold
    let mut right = vec![1.5f32; 128];

    strip.process_block(&mut left, &mut right, 1.0);

    // Verify soft clipper clamped the peak below 1.5
    for s in left.iter().chain(right.iter()) {
        assert!(*s < 1.45, "Clipper did not attenuate signal: got {}", *s);
        assert!(*s > 0.0);
    }
}
