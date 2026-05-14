use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct FlixConfig {
    pub default_install_path: Option<PathBuf>,
    pub packages: BTreeMap<String, PackageEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PackageEntry {
    pub source: String,
    pub tags: Vec<String>,
    pub version_hash: String,
    pub bin_path: PathBuf,
}

pub fn get_config_paths() -> (PathBuf, PathBuf) {
    let system_base = PathBuf::from("/usr/local/flix");
    
    // Check if system-wide install exists
    if system_base.exists() || std::env::var("USER").unwrap_or_default() == "root" {
        let etc_dir = system_base.join("etc");
        (etc_dir.join("config.toml"), etc_dir)
    } else if let Some(proj_dirs) = ProjectDirs::from("", "", "flix") {
        let user_config_dir = proj_dirs.config_dir().to_path_buf();
        (user_config_dir.join("config.toml"), user_config_dir)
    } else {
        (PathBuf::from(".flix_config.toml"), PathBuf::from("."))
    }
}

pub fn load_config() -> FlixConfig {
    let (path, _) = get_config_paths();
    if path.exists() {
        let contents = fs::read_to_string(path).unwrap_or_default();
        toml::from_str(&contents).unwrap_or_default()
    } else {
        FlixConfig::default()
    }
}

pub fn save_config(config: &FlixConfig) {
    let (path, dir) = get_config_paths();
    let toml_string = toml::to_string(config).expect("Failed to serialize config");
    
    if fs::create_dir_all(&dir).is_err() || fs::write(&path, &toml_string).is_err() {
        let tmp_path = std::env::temp_dir().join("flix_config.tmp");
        let _ = fs::write(&tmp_path, toml_string);
        let _ = std::process::Command::new("sudo").arg("mkdir").arg("-p").arg(&dir).status();
        let _ = std::process::Command::new("sudo").arg("cp").arg(&tmp_path).arg(&path).status();
    }
}

pub fn interactive_setup() -> PathBuf {
    println!("\nWelcome to Flix! Initial Setup Required.");
    println!("-----------------------------------------");
    
    println!("[1] System-wide (Default) ");
    println!("    - Location: /usr/local/flix/bin");
    
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home".into());
    let user_bin = format!("{}/.local/flix/bin", home);
    println!("[2] User-local");
    println!("    - Location: {}", user_bin);
    
    println!("[3] Custom Path");
    println!("[0] Cancel");

    print!("\nSelection [1]: ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let choice = input.trim();

    match choice {
        "2" => PathBuf::from(user_bin),
        "3" => {
            print!("Enter custom bin directory: ");
            io::stdout().flush().unwrap();
            let mut custom = String::new();
            io::stdin().read_line(&mut custom).unwrap();
            PathBuf::from(custom.trim())
        }
        "0" => std::process::exit(0),
        _ => PathBuf::from("/usr/local/flix/bin"),
    }
}
