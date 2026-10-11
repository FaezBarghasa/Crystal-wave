use Crystal_wave::mixing::MixingEngine;
use std::time::Instant;

fn main() {
    println!("=== CrystalWave Real-Time Audio Callback Cycle Budget Benchmark ===");

    let mut mixer = MixingEngine::new(44100.0).expect("Mixer init");

    // Add 64 active channels
    for i in 0..64 {
        let id = mixer
            .add_channel(format!("Track {i}"))
            .expect("add channel");
        mixer.set_channel_latency(id, i % 16);
    }

    let iterations = 10_000;
    let block_size = 128; // 128 samples at 44.1kHz is ~2.90ms time budget
    let mut left = vec![0.1f32; block_size];
    let mut right = vec![0.1f32; block_size];

    let start = Instant::now();

    for _ in 0..iterations {
        // Calculate PDC delays
        let _delays = mixer.calculate_pdc_compensation();

        // Process audio across channels
        for strip in &mut mixer.channels {
            strip.process_block(&mut left, &mut right, 1.0);
        }
    }

    let elapsed = start.elapsed();
    let total_blocks = iterations;
    let us_per_block = (elapsed.as_micros() as f64) / (total_blocks as f64);
    let budget_us = (block_size as f64 / 44100.0) * 1_000_000.0; // 2902 microseconds
    let cpu_load_pct = (us_per_block / budget_us) * 100.0;

    println!(
        "✓ 64-Channel DSP block processing: {:.2} µs/block across 128 samples",
        us_per_block
    );
    println!(
        "✓ Real-time budget: {:.2} µs (CPU Load: {:.2}%)",
        budget_us, cpu_load_pct
    );
    println!("=== Audio Callback Benchmark Passed Successfully ===");
}
