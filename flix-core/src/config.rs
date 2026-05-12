use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct FlixConfig {
    pub default_install_path: Option<PathBuf>,
    pub packages: BTreeMap<String, PackageEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PackageEntry {
    pub source: String,
    pub tags: Vec<String>,
    pub generation: u32,
    pub bin_path: PathBuf,
}

pub fn get_config_path() -> PathBuf {
    let system_flix = PathBuf::from("/usr/local/flix");
    let system_config = PathBuf::from("/usr/local/etc/flix/config.toml");

    if system_flix.exists() || std::env::var("USER").unwrap_or_default() == "root" {
        system_config
    } else if let Some(proj_dirs) = ProjectDirs::from("", "", "flix") {
        proj_dirs.config_dir().join("config.toml")
    } else {
        PathBuf::from(".flix_config.toml")
    }
}

pub fn load_config() -> FlixConfig {
    let path = get_config_path();
    if path.exists() {
        let contents = fs::read_to_string(path).unwrap_or_default();
        toml::from_str(&contents).unwrap_or_default()
    } else {
        FlixConfig::default()
    }
}

pub fn save_config(config: &FlixConfig) {
    let path = get_config_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let toml_string = toml::to_string(config).expect("Failed to serialize config");
    if let Err(_) = fs::write(&path, &toml_string) {
        let tmp_path = std::env::temp_dir().join("flix_config.tmp");
        fs::write(&tmp_path, toml_string).unwrap();
        let _ = std::process::Command::new("sudo").arg("mkdir").arg("-p").arg(path.parent().unwrap()).status();
        let _ = std::process::Command::new("sudo").arg("cp").arg(&tmp_path).arg(&path).status();
    }
}

pub fn enforce_flix_dir(mut path: PathBuf) -> PathBuf {
    if !path.ends_with("flix") {
        path.push("flix");
    }
    path
}
