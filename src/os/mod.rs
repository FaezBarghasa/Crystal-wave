/// Cross-platform real-time audio thread priority elevation manager
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadPriorityStatus {
    ElevatedRealtime,
    ElevatedHigh,
    Unchanged(i32),
    Unsupported,
}

pub struct ThreadPriorityManager;

impl ThreadPriorityManager {
    /// Elevates the current calling thread to real-time audio priority
    pub fn elevate_current_thread_to_realtime() -> ThreadPriorityStatus {
        #[cfg(target_os = "linux")]
        {
            unsafe {
                let mut param: libc::sched_param = std::mem::zeroed();
                param.sched_priority = 85; // Standard Linux pro-audio real-time priority
                let res = libc::sched_setscheduler(0, libc::SCHED_FIFO, &param);
                if res == 0 {
                    ThreadPriorityStatus::ElevatedRealtime
                } else {
                    // Fallback to renicing if SCHED_FIFO fails without CAP_SYS_NICE
                    let nice_res = libc::nice(-15);
                    if nice_res != -1 {
                        ThreadPriorityStatus::ElevatedHigh
                    } else {
                        ThreadPriorityStatus::Unchanged(res)
                    }
                }
            }
        }

        #[cfg(target_os = "windows")]
        {
            unsafe {
                use windows_sys::Win32::System::Threading::{
                    GetCurrentThread, SetThreadPriority, THREAD_PRIORITY_TIME_CRITICAL,
                };
                let handle = GetCurrentThread();
                let success = SetThreadPriority(handle, THREAD_PRIORITY_TIME_CRITICAL);
                if success != 0 {
                    ThreadPriorityStatus::ElevatedRealtime
                } else {
                    ThreadPriorityStatus::Unchanged(0)
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            unsafe {
                use mach2::mach_init::mach_thread_self;
                use mach2::thread_policy::{
                    THREAD_TIME_CONSTRAINT_POLICY, THREAD_TIME_CONSTRAINT_POLICY_COUNT,
                    thread_policy_set, thread_time_constraint_policy_data_t,
                };

                let thread_port = mach_thread_self();
                let mut policy: thread_time_constraint_policy_data_t = std::mem::zeroed();
                // 128 samples at 44.1kHz ~ 2.9ms period
                policy.period = 2_900_000;
                policy.computation = 1_500_000;
                policy.constraint = 2_900_000;
                policy.preemptible = 1;

                let kr = thread_policy_set(
                    thread_port,
                    THREAD_TIME_CONSTRAINT_POLICY,
                    &mut policy as *mut _ as *mut _,
                    THREAD_TIME_CONSTRAINT_POLICY_COUNT,
                );

                if kr == 0 {
                    ThreadPriorityStatus::ElevatedRealtime
                } else {
                    ThreadPriorityStatus::Unchanged(kr)
                }
            }
        }

        #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
        {
            ThreadPriorityStatus::Unsupported
        }
    }
}
