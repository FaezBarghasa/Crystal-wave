//! In-Memory & Memory-Mapped WAV Audio Buffer Pool
//!
//! Provides lock-free sample lookup for timeline audio clips loaded via `hound`.

use crate::audio::decoder::AudioDecoder;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone)]
pub struct PlanarStereoAudio {
    pub sample_rate: u32,
    pub left: Arc<Vec<f32>>,
    pub right: Arc<Vec<f32>>,
}

impl PlanarStereoAudio {
    pub fn len(&self) -> usize {
        self.left.len()
    }

    pub fn is_empty(&self) -> bool {
        self.left.is_empty()
    }

    #[inline]
    pub fn get_sample(&self, frame: usize) -> (f32, f32) {
        if frame < self.left.len() {
            (
                self.left[frame],
                self.right.get(frame).copied().unwrap_or(self.left[frame]),
            )
        } else {
            (0.0, 0.0)
        }
    }
}

/// Global shared WAV pool for zero-allocation real-time playback
#[derive(Clone, Default)]
pub struct WavBufferPool {
    cache: Arc<RwLock<HashMap<PathBuf, PlanarStereoAudio>>>,
}

impl WavBufferPool {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Load or retrieve decoded WAV audio by file path via hound
    pub fn get_or_load<P: AsRef<Path>>(&self, path: P) -> Result<PlanarStereoAudio, String> {
        let p = path.as_ref().to_path_buf();
        {
            let r = self.cache.read().map_err(|e| e.to_string())?;
            if let Some(buf) = r.get(&p) {
                return Ok(buf.clone());
            }
        }

        let decoded = AudioDecoder::decode_wav_file(&p)?;
        let (left, right) = decoded.to_planar_stereo();
        let planar = PlanarStereoAudio {
            sample_rate: decoded.sample_rate,
            left: Arc::new(left),
            right: Arc::new(right),
        };

        let mut w = self.cache.write().map_err(|e| e.to_string())?;
        w.insert(p, planar.clone());
        Ok(planar)
    }

    /// Insert raw in-memory WAV bytes (e.g. for synthetic takes or tests)
    pub fn insert_bytes(
        &self,
        key: impl Into<PathBuf>,
        wav_bytes: &[u8],
    ) -> Result<PlanarStereoAudio, String> {
        let decoded = AudioDecoder::decode_wav_bytes(wav_bytes)?;
        let (left, right) = decoded.to_planar_stereo();
        let planar = PlanarStereoAudio {
            sample_rate: decoded.sample_rate,
            left: Arc::new(left),
            right: Arc::new(right),
        };
        let mut w = self.cache.write().map_err(|e| e.to_string())?;
        w.insert(key.into(), planar.clone());
        Ok(planar)
    }

    /// Read sample safely for an audio file path without locking if already cached
    pub fn get_sample(&self, path: &Path, frame: usize) -> (f32, f32) {
        if let Ok(r) = self.cache.read()
            && let Some(audio) = r.get(path)
        {
            return audio.get_sample(frame);
        }
        (0.0, 0.0)
    }
}
