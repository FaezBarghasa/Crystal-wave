use crate::plugins::plugin_descriptor::{PluginCategory, PluginDescriptor};
use clack_host::entry::PluginEntry;
use std::env;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq)]
pub struct ClapPluginMetadata {
    pub id: String,
    pub name: String,
    pub vendor: String,
    pub url: String,
    pub version: String,
    pub description: String,
}

impl ClapPluginMetadata {
    pub fn to_descriptor(&self, numeric_id: u32) -> PluginDescriptor {
        let mut desc = PluginDescriptor::new(numeric_id, &self.name, PluginCategory::Utility);
        desc.version = self.version.clone();
        desc
    }
}

pub struct ClapBundleScanner;

impl ClapBundleScanner {
    /// Retrieve OS-standard CLAP plugin search directories
    pub fn standard_search_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // 1. Environment variable CLAP_PATH takes precedence
        if let Ok(val) = env::var("CLAP_PATH") {
            let separator = if cfg!(windows) { ';' } else { ':' };
            for part in val.split(separator) {
                if !part.trim().is_empty() {
                    paths.push(PathBuf::from(part.trim()));
                }
            }
        }

        // 2. User home and system standard locations
        if let Ok(home) = env::var("HOME") {
            let home_path = PathBuf::from(home);
            #[cfg(target_os = "macos")]
            {
                paths.push(home_path.join("Library/Audio/Plug-Ins/CLAP"));
                paths.push(PathBuf::from("/Library/Audio/Plug-Ins/CLAP"));
            }

            #[cfg(target_os = "linux")]
            {
                paths.push(home_path.join(".clap"));
                paths.push(PathBuf::from("/usr/lib/clap"));
                paths.push(PathBuf::from("/usr/local/lib/clap"));
            }
        }

        #[cfg(target_os = "windows")]
        {
            if let Ok(prog_files) = env::var("CommonProgramFiles") {
                paths.push(PathBuf::from(prog_files).join("CLAP"));
            }
        }

        paths
    }

    /// Scan a directory for .clap bundle files
    pub fn scan_directory(dir: &Path) -> Vec<PathBuf> {
        let mut bundles = Vec::new();
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path
                    .extension()
                    .is_some_and(|e| e.eq_ignore_ascii_case("clap"))
                {
                    bundles.push(path);
                }
            }
        }
        bundles
    }

    /// Inspect a CLAP bundle and extract plugin metadata using `clack-host`
    pub fn inspect_bundle(bundle_path: &Path) -> Result<Vec<ClapPluginMetadata>, String> {
        if !bundle_path.exists() {
            return Err(format!("File does not exist: {:?}", bundle_path));
        }

        // SAFETY: Loading dynamic libraries requires unsafe in clack-host / libloading.
        let entry = unsafe {
            PluginEntry::load(bundle_path)
                .map_err(|e| format!("Failed to load CLAP entry: {:?}", e))?
        };

        let factory = entry
            .get_plugin_factory()
            .ok_or_else(|| "CLAP bundle does not expose a PluginFactory".to_string())?;

        let count = factory.plugin_count();
        let mut results = Vec::with_capacity(count as usize);

        for i in 0..count {
            if let Some(desc) = factory.plugin_descriptor(i) {
                let id = desc
                    .id()
                    .and_then(|c| c.to_str().ok())
                    .unwrap_or("unknown")
                    .to_string();
                let name = desc
                    .name()
                    .and_then(|c| c.to_str().ok())
                    .unwrap_or("Unnamed CLAP Plugin")
                    .to_string();
                let vendor = desc
                    .vendor()
                    .and_then(|c| c.to_str().ok())
                    .unwrap_or("")
                    .to_string();
                let url = desc
                    .url()
                    .and_then(|c| c.to_str().ok())
                    .unwrap_or("")
                    .to_string();
                let version = desc
                    .version()
                    .and_then(|c| c.to_str().ok())
                    .unwrap_or("1.0.0")
                    .to_string();
                let description = desc
                    .description()
                    .and_then(|c| c.to_str().ok())
                    .unwrap_or("")
                    .to_string();

                results.push(ClapPluginMetadata {
                    id,
                    name,
                    vendor,
                    url,
                    version,
                    description,
                });
            }
        }

        Ok(results)
    }
}
