use Crystal_wave::audio::realtime_gc::RealtimeBufferRecycler;
use std::thread;

#[test]
fn test_realtime_buffer_deferred_deallocation() {
    let mut recycler = RealtimeBufferRecycler::new();
    let handle = recycler.handle();

    assert_eq!(recycler.total_allocated(), 0);
    assert_eq!(recycler.total_collected(), 0);

    // Allocate 10 buffers
    let mut buffers = Vec::new();
    for _ in 0..10 {
        let mut buf = handle.allocate(512);
        buf[0] = 1.0;
        buf[511] = -1.0;
        buffers.push(buf);
    }
    assert_eq!(recycler.total_allocated(), 10);

    // Spawn a simulated real-time audio thread that processes and drops the buffers
    let handle_thread = thread::spawn(move || {
        for mut buf in buffers {
            // Simulated audio block processing
            for sample in buf.iter_mut() {
                *sample *= 0.5;
            }
            // Drop buffer in the simulated audio thread
            drop(buf);
        }
    });

    handle_thread.join().expect("Audio thread failed");

    // Before collection, background collection count is 0
    assert_eq!(recycler.total_collected(), 0);

    // Background thread collects and frees the dropped nodes
    let freed = recycler.collect();
    assert_eq!(freed, 10);
    assert_eq!(recycler.total_collected(), 10);

    // Subsequent collect on empty queue yields 0
    let second_pass = recycler.collect();
    assert_eq!(second_pass, 0);
}
