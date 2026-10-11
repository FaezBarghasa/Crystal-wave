use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{OutputCallbackInfo, Stream, StreamConfig};
use rtrb::{Producer, RingBuffer};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

pub struct AudioHardwareDriver {
    pub host_name: String,
    pub device_name: String,
    pub sample_rate: u32,
    pub channels: u16,
    pub is_running: Arc<AtomicBool>,
    pub underrun_count: Arc<AtomicU64>,
    stream: Option<Stream>,
    pub audio_producer: Option<Producer<f32>>,
}

impl AudioHardwareDriver {
    pub fn new() -> Result<Self, String> {
        let host = cpal::default_host();
        let host_name = host.id().name().to_string();

        let device = host
            .default_output_device()
            .ok_or_else(|| "No default audio output device available".to_string())?;
        let device_name = device.to_string();

        let default_config = device
            .default_output_config()
            .map_err(|e| format!("Failed to query default output config: {}", e))?;

        let sample_rate = default_config.sample_rate();
        let channels = default_config.channels();

        Ok(Self {
            host_name,
            device_name,
            sample_rate,
            channels,
            is_running: Arc::new(AtomicBool::new(false)),
            underrun_count: Arc::new(AtomicU64::new(0)),
            stream: None,
            audio_producer: None,
        })
    }

    /// Initializes a lock-free SPSC audio output stream bridged to the hardware DAC.
    pub fn start_stream(&mut self, buffer_capacity_frames: usize) -> Result<Producer<f32>, String> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| "No default audio output device".to_string())?;

        let config: StreamConfig = device
            .default_output_config()
            .map_err(|e| format!("Config query error: {}", e))?
            .into();

        let total_samples = buffer_capacity_frames * (config.channels as usize);
        let (producer, mut consumer) = RingBuffer::<f32>::new(total_samples);

        let underrun_counter = Arc::clone(&self.underrun_count);
        let running_flag = Arc::clone(&self.is_running);
        running_flag.store(true, Ordering::SeqCst);

        let err_fn = move |err| {
            eprintln!("Audio DAC Stream Error: {}", err);
        };

        let stream = device
            .build_output_stream(
                config,
                move |data: &mut [f32], _: &OutputCallbackInfo| {
                    crate::os::ThreadPriorityManager::enable_ftz_daz();
                    let needed = data.len();
                    let available = consumer.slots();
                    let to_read = available.min(needed);

                    if to_read > 0
                        && let Ok(chunk) = consumer.read_chunk(to_read)
                    {
                        let (s1, s2) = chunk.as_slices();
                        let s1_len = s1.len();
                        data[..s1_len].copy_from_slice(s1);
                        data[s1_len..to_read].copy_from_slice(s2);
                        chunk.commit_all();
                    }

                    if to_read < needed {
                        data[to_read..].fill(0.0);
                        underrun_counter.fetch_add(1, Ordering::Relaxed);
                    }
                },
                err_fn,
                None,
            )
            .map_err(|e| format!("Failed to build audio output stream: {}", e))?;

        stream
            .play()
            .map_err(|e| format!("Failed to play stream: {}", e))?;

        self.stream = Some(stream);
        Ok(producer)
    }

    pub fn pause(&mut self) -> Result<(), String> {
        if let Some(ref stream) = self.stream {
            stream
                .pause()
                .map_err(|e| format!("Failed to pause stream: {}", e))?;
            self.is_running.store(false, Ordering::SeqCst);
        }
        Ok(())
    }

    pub fn resume(&mut self) -> Result<(), String> {
        if let Some(ref stream) = self.stream {
            stream
                .play()
                .map_err(|e| format!("Failed to resume stream: {}", e))?;
            self.is_running.store(true, Ordering::SeqCst);
        }
        Ok(())
    }

    pub fn is_active(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }

    pub fn underruns(&self) -> u64 {
        self.underrun_count.load(Ordering::Relaxed)
    }
}
