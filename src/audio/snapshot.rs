//! Lock-Free Audio Engine Project Snapshot (`arc-swap`)
//!
//! Provides atomic, wait-free project snapshot publishing from the UI/Control
//! thread to the real-time audio thread callback without priority inversion or mutexes.

use crate::fl_studio::FlPianoRollNote;
use crate::timeline::track::Track;
use std::sync::Arc;

/// Immutable snapshot of a mixer channel strip state for lock-free audio processing
#[derive(Debug, Clone)]
pub struct ChannelStripSnapshot {
    pub id: usize,
    pub volume_fader_db: f32,
    pub pan_balance: f32,
    pub muted: bool,
    pub solo: bool,
    pub phase_invert: bool,
    pub swap_channels: bool,
    pub output_bus: usize,
    pub vca_group: Option<usize>,
    pub latency_samples: usize,
}

/// Immutable snapshot of the entire project audio state read by the real-time audio thread
#[derive(Debug, Clone)]
pub struct AudioProjectSnapshot {
    pub tracks: Vec<Track>,
    pub piano_roll_notes: Vec<FlPianoRollNote>,
    pub mixer_channels: Vec<ChannelStripSnapshot>,
    pub master_volume_fader_db: f32,
    pub master_soft_clipper_enabled: bool,
    pub mixer_sends: Vec<Vec<f32>>,
    pub tempo_bpm: f32,
    pub sample_rate: f32,
    pub swing_amount: f32,
    pub pdc_enabled: bool,
    pub pdc_delays: Vec<usize>,
}

impl Default for AudioProjectSnapshot {
    fn default() -> Self {
        Self {
            tracks: Vec::new(),
            piano_roll_notes: Vec::new(),
            mixer_channels: Vec::new(),
            master_volume_fader_db: 0.0,
            master_soft_clipper_enabled: true,
            mixer_sends: vec![vec![0.0; 128]; 128],
            tempo_bpm: 120.0,
            sample_rate: 44100.0,
            swing_amount: 0.0,
            pdc_enabled: true,
            pdc_delays: Vec::new(),
        }
    }
}

/// Thread-safe lock-free container for publishing and consuming audio project snapshots
#[derive(Clone)]
pub struct AudioSnapshotHandle {
    inner: Arc<arc_swap::ArcSwap<AudioProjectSnapshot>>,
}

impl Default for AudioSnapshotHandle {
    fn default() -> Self {
        Self::new(AudioProjectSnapshot::default())
    }
}

impl AudioSnapshotHandle {
    pub fn new(initial: AudioProjectSnapshot) -> Self {
        Self {
            inner: Arc::new(arc_swap::ArcSwap::from_pointee(initial)),
        }
    }

    /// UI/Control thread: Publish updated project state atomically without blocking
    pub fn store(&self, snapshot: AudioProjectSnapshot) {
        self.inner.store(Arc::new(snapshot));
    }

    /// Real-time audio thread: Lock-free read of latest snapshot
    pub fn load(&self) -> arc_swap::Guard<Arc<AudioProjectSnapshot>> {
        self.inner.load()
    }

    /// Real-time audio thread: Load Arc pointer directly
    pub fn load_full(&self) -> Arc<AudioProjectSnapshot> {
        self.inner.load_full()
    }
}
