use Crystal_wave::os::{ThreadPriorityManager, ThreadPriorityStatus};

#[test]
fn test_cross_platform_thread_priority_elevation() {
    let status = ThreadPriorityManager::elevate_current_thread_to_realtime();
    println!("Thread priority elevation result: {:?}", status);

    // On Linux/macOS/Windows, status will be either ElevatedRealtime, ElevatedHigh, or Unchanged(code)
    // depending on process permissions (e.g. CAP_SYS_NICE on Linux), but must never panic.
    match status {
        ThreadPriorityStatus::ElevatedRealtime => {
            println!("✓ Successfully elevated to real-time priority");
        }
        ThreadPriorityStatus::ElevatedHigh => {
            println!("✓ Successfully reniced to high priority fallback");
        }
        ThreadPriorityStatus::Unchanged(err) => {
            println!("ℹ Permission limit for real-time elevation: {}", err);
        }
        ThreadPriorityStatus::Unsupported => {
            println!("ℹ Platform does not support thread priority elevation");
        }
    }
}
