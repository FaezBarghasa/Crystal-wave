//! Dedicated OS Real-Time Audio Worker (`cw-audio-engine`)
//!
//! Completely decouples real-time audio block processing from the 60 Hz GUI tick,
//! eliminating DAC starvation, audio glitching, and false-positive XRUNs.

use crate::audio::playback_engine::TimelineAudioScheduler;
use crate::audio::snapshot::AudioSnapshotHandle;
use crate::mixing::MixingEngine;
use rtrb::{Producer, RingBuffer};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

/// Commands sent from the UI/Control thread to the real-time audio worker via SPSC ringbuffer
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AudioWorkerCommand {
    Play,
    Pause,
    Stop,
    Seek(u64),
    SetTempo(f32),
    SetSwing(f32),
    Shutdown,
}

/// Shared real-time atomic telemetry and transport state
#[derive(Debug, Clone)]
pub struct AudioWorkerSharedState {
    pub playhead_frame: Arc<AtomicU64>,
    pub is_playing: Arc<AtomicBool>,
    pub cpu_load_percent: Arc<AtomicU32>, // fixed-point: value * 10 (e.g. 25 = 2.5%)
    pub underruns: Arc<AtomicU64>,
    pub tempo_bpm: Arc<AtomicU32>, // fixed-point: bpm * 100 (e.g. 12000 = 120.00)
    pub swing_percent: Arc<AtomicU32>, // 0..100
}

impl Default for AudioWorkerSharedState {
    fn default() -> Self {
        Self {
            playhead_frame: Arc::new(AtomicU64::new(0)),
            is_playing: Arc::new(AtomicBool::new(false)),
            cpu_load_percent: Arc::new(AtomicU32::new(0)),
            underruns: Arc::new(AtomicU64::new(0)),
            tempo_bpm: Arc::new(AtomicU32::new(12000)),
            swing_percent: Arc::new(AtomicU32::new(0)),
        }
    }
}

/// Handle to communicate with the background real-time audio worker thread
pub struct AudioWorkerHandle {
    pub command_producer: Producer<AudioWorkerCommand>,
    pub snapshot_handle: AudioSnapshotHandle,
    pub state: AudioWorkerSharedState,
    thread_handle: Option<JoinHandle<()>>,
    shutdown_flag: Arc<AtomicBool>,
}

impl AudioWorkerHandle {
    /// Send play command to worker
    pub fn play(&mut self) {
        let _ = self.command_producer.push(AudioWorkerCommand::Play);
        self.state.is_playing.store(true, Ordering::SeqCst);
    }

    /// Send pause command to worker
    pub fn pause(&mut self) {
        let _ = self.command_producer.push(AudioWorkerCommand::Pause);
        self.state.is_playing.store(false, Ordering::SeqCst);
    }

    /// Send stop command to worker and reset playhead
    pub fn stop(&mut self) {
        let _ = self.command_producer.push(AudioWorkerCommand::Stop);
        self.state.is_playing.store(false, Ordering::SeqCst);
        self.state.playhead_frame.store(0, Ordering::SeqCst);
    }

    /// Seek to exact audio frame
    pub fn seek(&mut self, frame: u64) {
        let _ = self.command_producer.push(AudioWorkerCommand::Seek(frame));
        self.state.playhead_frame.store(frame, Ordering::SeqCst);
    }

    /// Update musical tempo
    pub fn set_tempo(&mut self, bpm: f32) {
        let _ = self
            .command_producer
            .push(AudioWorkerCommand::SetTempo(bpm));
        self.state
            .tempo_bpm
            .store((bpm * 100.0).max(100.0) as u32, Ordering::SeqCst);
    }

    /// Update FL Studio swing parameter (0.0 to 1.0)
    pub fn set_swing(&mut self, swing: f32) {
        let _ = self
            .command_producer
            .push(AudioWorkerCommand::SetSwing(swing));
        self.state
            .swing_percent
            .store((swing * 100.0).clamp(0.0, 100.0) as u32, Ordering::SeqCst);
    }

    /// Read atomic playhead frame
    pub fn playhead_frame(&self) -> u64 {
        self.state.playhead_frame.load(Ordering::Relaxed)
    }

    /// Check if audio worker is actively playing
    pub fn is_playing(&self) -> bool {
        self.state.is_playing.load(Ordering::Relaxed)
    }

    /// Read calculated DSP load percentage (0.0% to 100.0%)
    pub fn cpu_load(&self) -> f32 {
        self.state.cpu_load_percent.load(Ordering::Relaxed) as f32 / 10.0
    }

    /// Read underrun count
    pub fn underruns(&self) -> u64 {
        self.state.underruns.load(Ordering::Relaxed)
    }
}

impl Drop for AudioWorkerHandle {
    fn drop(&mut self) {
        self.shutdown_flag.store(true, Ordering::SeqCst);
        let _ = self.command_producer.push(AudioWorkerCommand::Shutdown);
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}

/// Spawns the dedicated OS thread named `cw-audio-engine`
pub fn spawn_audio_worker(
    sample_rate: f32,
    block_frames: usize,
    mut dac_producer: Option<Producer<f32>>,
    snapshot_handle: AudioSnapshotHandle,
) -> AudioWorkerHandle {
    let (command_producer, mut command_consumer) = RingBuffer::<AudioWorkerCommand>::new(64);
    let state = AudioWorkerSharedState::default();
    let shutdown_flag = Arc::new(AtomicBool::new(false));

    let worker_state = state.clone();
    let worker_shutdown = Arc::clone(&shutdown_flag);
    let worker_snapshot = snapshot_handle.clone();

    let thread_handle = thread::Builder::new()
        .name("cw-audio-engine".to_string())
        .spawn(move || {
            // Priority elevation and FTZ/DAZ mode
            crate::os::ThreadPriorityManager::elevate_current_thread_to_realtime();
            crate::os::ThreadPriorityManager::enable_ftz_daz();

            let block_size = block_frames.max(64);
            let block_duration =
                Duration::from_secs_f64(block_size as f64 / sample_rate.max(8000.0) as f64);

            let mut scheduler = TimelineAudioScheduler::new(sample_rate, 120.0);
            let mut mixer = MixingEngine::new(sample_rate).unwrap_or_else(|_| {
                MixingEngine::new(44100.0).expect("Fallback mixer creation")
            });
            let mut master_buf_l = vec![0.0f32; block_size];
            let mut master_buf_r = vec![0.0f32; block_size];

            while !worker_shutdown.load(Ordering::Relaxed) {
                // Drain incoming commands
                while let Ok(cmd) = command_consumer.pop() {
                    match cmd {
                        AudioWorkerCommand::Play => {
                            worker_state.is_playing.store(true, Ordering::SeqCst);
                        }
                        AudioWorkerCommand::Pause => {
                            worker_state.is_playing.store(false, Ordering::SeqCst);
                        }
                        AudioWorkerCommand::Stop => {
                            worker_state.is_playing.store(false, Ordering::SeqCst);
                            worker_state.playhead_frame.store(0, Ordering::SeqCst);
                        }
                        AudioWorkerCommand::Seek(frame) => {
                            worker_state.playhead_frame.store(frame, Ordering::SeqCst);
                        }
                        AudioWorkerCommand::SetTempo(bpm) => {
                            worker_state
                                .tempo_bpm
                                .store((bpm * 100.0).max(100.0) as u32, Ordering::SeqCst);
                            scheduler.set_tempo(bpm);
                        }
                        AudioWorkerCommand::SetSwing(swing) => {
                            worker_state
                                .swing_percent
                                .store((swing * 100.0).clamp(0.0, 100.0) as u32, Ordering::SeqCst);
                            scheduler.set_swing(swing);
                        }
                        AudioWorkerCommand::Shutdown => {
                            return;
                        }
                    }
                }

                if worker_state.is_playing.load(Ordering::Relaxed) {
                    let start_time = Instant::now();
                    let current_playhead = worker_state.playhead_frame.load(Ordering::Relaxed);

                    // Load lock-free project audio snapshot via ArcSwap
                    let snap = worker_snapshot.load();
                    scheduler.set_tempo(snap.tempo_bpm);
                    scheduler.set_swing(snap.swing_amount);

                    // Sync mixer state from snapshot
                    mixer.master_strip.volume_fader_db = snap.master_volume_fader_db;
                    mixer.master_soft_clipper_enabled = snap.master_soft_clipper_enabled;
                    mixer.pdc_enabled = snap.pdc_enabled;

                    // Sync channels
                    for ch_snap in &snap.mixer_channels {
                        let _ = mixer.ensure_channel_exists(ch_snap.id);
                        if let Some(strip) = mixer.channels.get_mut(ch_snap.id) {
                            strip.volume_fader_db = ch_snap.volume_fader_db;
                            strip.pan.balance = ch_snap.pan_balance;
                            strip.muted = ch_snap.muted;
                            strip.solo = ch_snap.solo;
                            strip.phase_invert = ch_snap.phase_invert;
                            strip.swap_channels = ch_snap.swap_channels;
                            strip.output_bus = ch_snap.output_bus;
                            strip.vca_group = ch_snap.vca_group;
                        }
                        mixer.set_channel_latency(ch_snap.id, ch_snap.latency_samples);
                    }

                    // Process sample-accurate audio block through timeline scheduler and mixer graph
                    master_buf_l.fill(0.0);
                    master_buf_r.fill(0.0);
                    scheduler.process_block(
                        &snap.tracks,
                        &snap.piano_roll_notes,
                        &mut mixer,
                        current_playhead,
                        block_size,
                        &mut master_buf_l,
                        &mut master_buf_r,
                    );

                    // If audio DAC producer exists, push stereo samples to CPAL ringbuffer
                    if let Some(ref mut prod) = dac_producer {
                        let slots_needed = block_size * 2;
                        if prod.slots() >= slots_needed {
                            for i in 0..block_size {
                                let _ = prod.push(master_buf_l[i]);
                                let _ = prod.push(master_buf_r[i]);
                            }
                        }
                    }

                    worker_state
                        .playhead_frame
                        .fetch_add(block_size as u64, Ordering::Relaxed);

                    let elapsed = start_time.elapsed();
                    let dsp_fraction =
                        (elapsed.as_secs_f64() / block_duration.as_secs_f64()) * 100.0;
                    worker_state
                        .cpu_load_percent
                        .store((dsp_fraction * 10.0) as u32, Ordering::Relaxed);

                    // Rate-limit to real-time block clock
                    if elapsed < block_duration {
                        thread::sleep(block_duration - elapsed);
                    }
                } else {
                    // Sleep short duration when idle to keep CPU usage low
                    thread::sleep(Duration::from_millis(5));
                }
            }
        })
        .expect("Failed to spawn cw-audio-engine worker thread");

    AudioWorkerHandle {
        command_producer,
        snapshot_handle,
        state,
        thread_handle: Some(thread_handle),
        shutdown_flag,
    }
}
