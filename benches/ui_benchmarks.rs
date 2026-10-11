use std::time::Instant;
use Crystal_wave::ui::message::UiMessage;
use Crystal_wave::ui::session_driver::UiSessionDriver;
use Crystal_wave::ui::telemetry::{ChannelMeterFrame, TelemetryBatch, TelemetryChannel};

fn main() {
    println!("=== CrystalWave UI Performance & Profiling Benchmarks ===");

    // Benchmark 1: Pure update() state reducer throughput (100,000 messages)
    let mut driver = UiSessionDriver::new(44100.0);
    let iterations = 100_000;
    let start = Instant::now();

    for i in 0..iterations {
        let fader_db = -24.0 + ((i % 40) as f32);
        driver.dispatch(UiMessage::SetTrackFader {
            track_id: 0,
            volume_db: fader_db,
        });
        driver.dispatch(UiMessage::SetPlayhead(i as u64));
    }

    let elapsed = start.elapsed();
    let total_ops = iterations * 2;
    let ops_per_sec = (total_ops as f64) / elapsed.as_secs_f64();
    let ns_per_op = (elapsed.as_nanos() as f64) / (total_ops as f64);

    println!(
        "✓ UI update() throughput: {:?} for {} ops ({:.2} M ops/sec, {:.2} ns/op)",
        elapsed,
        total_ops,
        ops_per_sec / 1_000_000.0,
        ns_per_op
    );

    // Benchmark 2: SPSC rtrb telemetry throughput (100,000 batches)
    let (mut producer, mut consumer) = TelemetryChannel::pair(1024);
    let batch = TelemetryBatch {
        playhead_tick: 1920,
        cpu_load_percent: 18.4,
        meters: vec![ChannelMeterFrame {
            channel_id: 0,
            peak_db: -6.0,
            rms_db: -18.0,
            auto_mute_tripped: false,
        }],
    };

    let start = Instant::now();
    for _ in 0..iterations {
        if producer.push(batch.clone()).is_ok() {
            let _ = consumer.pop();
        }
    }
    let elapsed = start.elapsed();
    let spsc_ops_per_sec = (iterations as f64) / elapsed.as_secs_f64();

    println!(
        "✓ Lock-free SPSC telemetry throughput: {:?} for {} batches ({:.2} M batches/sec)",
        elapsed,
        iterations,
        spsc_ops_per_sec / 1_000_000.0
    );

    println!("=== All Benchmarks Completed Successfully ===");
}
