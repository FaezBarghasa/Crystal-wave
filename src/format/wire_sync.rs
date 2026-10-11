use serde::{Deserialize, Serialize};

/// High-throughput QUIC datagram frame for streaming real-time audio over Edgeray
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EdgeRayAudioPacket {
    pub sequence_number: u64,
    pub timestamp_nanos: u64,
    pub sample_rate: u32,
    pub channels: u16,
    pub frame_count: u32,
    pub payload_f32: Vec<f32>,
}

impl EdgeRayAudioPacket {
    pub fn new(
        sequence_number: u64,
        timestamp_nanos: u64,
        sample_rate: u32,
        channels: u16,
        samples: &[f32],
    ) -> Self {
        let frame_count = (samples.len() / (channels as usize).max(1)) as u32;
        Self {
            sequence_number,
            timestamp_nanos,
            sample_rate,
            channels,
            frame_count,
            payload_f32: samples.to_vec(),
        }
    }

    /// Encode packet to compact binary representation (JSON or raw binary)
    pub fn to_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }
}

/// Lightweight edge telemetry datagram for live multi-seat playhead and meter synchronization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EdgeRayTelemetryDatagram {
    pub session_id: String,
    pub playhead_tick: u64,
    pub bpm: f32,
    pub is_playing: bool,
    pub cpu_load_percent: f32,
    pub master_peak_db: f32,
    pub master_rms_db: f32,
}

impl EdgeRayTelemetryDatagram {
    pub fn to_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }
}

/// Crysta-backend session interchange envelope for cloud sync & SurrealDB persistence
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CrystaSessionEnvelope {
    pub session_id: String,
    pub project_name: String,
    pub version: u64,
    pub author_paseto_id: String,
    pub sample_rate: f32,
    pub bpm: f32,
    pub tracks_count: usize,
    pub serialized_manifest_json: String,
}

impl CrystaSessionEnvelope {
    pub fn new(
        session_id: impl Into<String>,
        project_name: impl Into<String>,
        author_id: impl Into<String>,
        sample_rate: f32,
        bpm: f32,
        tracks_count: usize,
        manifest_json: impl Into<String>,
    ) -> Self {
        Self {
            session_id: session_id.into(),
            project_name: project_name.into(),
            version: 1,
            author_paseto_id: author_id.into(),
            sample_rate,
            bpm,
            tracks_count,
            serialized_manifest_json: manifest_json.into(),
        }
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }
}
