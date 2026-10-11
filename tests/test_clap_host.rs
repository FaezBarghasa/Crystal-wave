use Crystal_wave::plugins::clap_host::{ClapBundleScanner, ClapPluginMetadata};
use std::env;
use std::fs::File;
use std::path::PathBuf;

#[test]
fn test_clap_bundle_standard_paths() {
    let original = env::var("CLAP_PATH").ok();
    // SAFETY: Single-threaded test context modifying test env var
    unsafe {
        env::set_var("CLAP_PATH", "/tmp/custom_clap_dir:/opt/clap_plugins");
    }

    let paths = ClapBundleScanner::standard_search_paths();
    assert!(paths.contains(&PathBuf::from("/tmp/custom_clap_dir")));
    assert!(paths.contains(&PathBuf::from("/opt/clap_plugins")));

    // Restore env var
    unsafe {
        match original {
            Some(v) => env::set_var("CLAP_PATH", v),
            None => env::remove_var("CLAP_PATH"),
        }
    }
}

#[test]
fn test_clap_bundle_directory_scanning() {
    let temp_dir = env::temp_dir().join("crystalwave_clap_test");
    let _ = std::fs::create_dir_all(&temp_dir);

    // Create dummy files
    let dummy_clap = temp_dir.join("synth.clap");
    let dummy_txt = temp_dir.join("readme.txt");
    let _ = File::create(&dummy_clap);
    let _ = File::create(&dummy_txt);

    let found = ClapBundleScanner::scan_directory(&temp_dir);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0], dummy_clap);

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_clap_metadata_to_descriptor_conversion() {
    let meta = ClapPluginMetadata {
        id: "com.crystalwave.delay".to_string(),
        name: "Crystal Stereo Delay".to_string(),
        vendor: "CrystalWave DSP".to_string(),
        url: "https://crystalwave.audio".to_string(),
        version: "2.1.0".to_string(),
        description: "Zero-latency broadcast delay effect".to_string(),
    };

    let desc = meta.to_descriptor(5001);
    assert_eq!(desc.unique_id, 5001);
    assert_eq!(desc.name, "Crystal Stereo Delay");
    assert_eq!(desc.version, "2.1.0");
}

#[test]
fn test_clap_inspect_nonexistent_bundle() {
    let non_existent = PathBuf::from("/does/not/exist/plugin.clap");
    let result = ClapBundleScanner::inspect_bundle(&non_existent);
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(err.contains("File does not exist"));
}
