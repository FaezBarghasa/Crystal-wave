use basedrop::{Collector, Handle, Owned};
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

/// A real-time safe audio buffer wrapped in a deferred collection smart pointer.
/// When dropped inside an audio callback, it never calls `free` or takes any locks.
pub struct RealtimeAudioBuffer {
    inner: Owned<Vec<f32>>,
}

impl Deref for RealtimeAudioBuffer {
    type Target = [f32];

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for RealtimeAudioBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl RealtimeAudioBuffer {
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn fill(&mut self, val: f32) {
        self.inner.fill(val);
    }
}

/// Handle held by producer threads or buffer pools to allocate real-time safe buffers.
#[derive(Clone)]
pub struct RealtimeBufferPoolHandle {
    handle: Handle,
    alloc_counter: Arc<AtomicUsize>,
}

impl RealtimeBufferPoolHandle {
    /// Allocate a new buffer of given capacity that is safe to drop on the audio thread.
    pub fn allocate(&self, samples: usize) -> RealtimeAudioBuffer {
        self.alloc_counter.fetch_add(1, Ordering::Relaxed);
        let vec = vec![0.0f32; samples];
        let owned = Owned::new(&self.handle, vec);
        RealtimeAudioBuffer { inner: owned }
    }
}

/// Garbage collector running on a non-realtime background or UI thread to reclaim audio buffers.
pub struct RealtimeBufferRecycler {
    collector: Collector,
    alloc_counter: Arc<AtomicUsize>,
    collect_counter: Arc<AtomicUsize>,
}

impl Default for RealtimeBufferRecycler {
    fn default() -> Self {
        Self::new()
    }
}

impl RealtimeBufferRecycler {
    pub fn new() -> Self {
        Self {
            collector: Collector::new(),
            alloc_counter: Arc::new(AtomicUsize::new(0)),
            collect_counter: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Obtain an allocation handle to be used by producer threads.
    pub fn handle(&self) -> RealtimeBufferPoolHandle {
        RealtimeBufferPoolHandle {
            handle: self.collector.handle(),
            alloc_counter: Arc::clone(&self.alloc_counter),
        }
    }

    /// Perform a non-realtime garbage collection pass to deallocate buffers dropped by the audio thread.
    pub fn collect(&mut self) -> usize {
        let mut count = 0;
        while self.collector.collect_one() {
            count += 1;
        }
        self.collect_counter.fetch_add(count, Ordering::Relaxed);
        count
    }

    /// Total number of buffers allocated since inception.
    pub fn total_allocated(&self) -> usize {
        self.alloc_counter.load(Ordering::Relaxed)
    }

    /// Total number of buffers freed on the background thread since inception.
    pub fn total_collected(&self) -> usize {
        self.collect_counter.load(Ordering::Relaxed)
    }
}
