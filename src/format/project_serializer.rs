use crate::format::project_format::CrystalWaveProject;

pub struct ProjectSerializer;

impl ProjectSerializer {
    /// Serializes project model into formatted JSON string
    pub fn to_json_string(project: &CrystalWaveProject) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(project)
    }

    /// Serializes project model into raw JSON byte vector
    pub fn to_json_bytes(project: &CrystalWaveProject) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(project)
    }
}
