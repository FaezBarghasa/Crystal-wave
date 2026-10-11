use crate::format::project_format::CrystalWaveProject;

pub struct ProjectLoader;

impl ProjectLoader {
    /// Safely deserializes project model from JSON string
    pub fn from_json_str(json: &str) -> Result<CrystalWaveProject, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Safely deserializes project model from byte slice
    pub fn from_bytes(bytes: &[u8]) -> Result<CrystalWaveProject, serde_json::Error> {
        serde_json::from_slice(bytes)
    }
}
