pub mod config {
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

    #[derive(Debug, Serialize, Deserialize)]
    pub struct PackageEntry {
        pub source: String,
        pub tags: Vec<String>,
        pub generation: u32,
        pub bin_path: PathBuf,
    }

    pub fn get_config_path() -> PathBuf {
        if let Some(proj_dirs) = ProjectDirs::from("", "", "flix") {
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
            fs::create_dir_all(parent).expect("Failed to create config directory");
        }
        let toml_string = toml::to_string(config).expect("Failed to serialize config");
        fs::write(path, toml_string).expect("Failed to write config file");
    }

    pub fn interactive_setup() -> PathBuf {
        println!("\n👋 Welcome to Flix! Let's set up your default installation directory.");
        println!("All packages will be stored in a dedicated 'flix' subfolder for organization.\n");
        
        println!("[1] System-wide (requires sudo): /usr/local/bin/flix");
        
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home".into());
        let user_default = format!("{}/.local/bin/flix", home);
        println!("[2] User-only: {}", user_default);
        
        println!("[3] Custom Path");
        println!("[0] Cancel");

        print!("\nSelection [Default: 1]: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let choice = input.trim();

        let path = match choice {
            "2" => PathBuf::from(user_default),
            "3" => {
                print!("Enter custom path: ");
                io::stdout().flush().unwrap();
                let mut custom = String::new();
                io::stdin().read_line(&mut custom).unwrap();
                let mut p = PathBuf::from(custom.trim());
                if !p.ends_with("flix") { p = p.join("flix"); }
                p
            }
            "0" => std::process::exit(0),
            _ => PathBuf::from("/usr/local/bin/flix"),
        };

        println!("✅ Default path set to: {}", path.display());
        path
    }
}

pub mod engine {
    use super::config::{load_config, save_config, PackageEntry, interactive_setup};
    use git2::Repository;
    use std::env;
    use std::fs;
    use std::io;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    pub fn install(
        url: &str,
        _release: bool, // Prefix with _ to silence warning until implemented
        _default: bool,
        _quiet: bool,
        yes: bool,
        tags: &[String],
        custom_path: Option<&str>,
    ) {
        let mut config = load_config();

        let install_dir = if let Some(p) = custom_path {
            let mut path = PathBuf::from(p);
            if !path.ends_with("flix") { path = path.join("flix"); }
            path
        } else if let Some(p) = config.default_install_path.clone() {
            p
        } else {
            if yes {
                PathBuf::from("/usr/local/bin/flix")
            } else {
                let chosen = interactive_setup();
                config.default_install_path = Some(chosen.clone());
                save_config(&config);
                chosen
            }
        };

        println!("🚀 Preparing to install to: {}", install_dir.display());
        ensure_dir_exists(&install_dir);

        let url_clean = url.trim_end_matches(".git");
        let package_name = url_clean.split('/').last().unwrap_or("app").to_string();

        let temp_dir = env::temp_dir().join("flix_builds").join(&package_name);
        if temp_dir.exists() { let _ = fs::remove_dir_all(&temp_dir); }

        println!("📦 Cloning repository...");
        if Repository::clone(url, &temp_dir).is_err() { return; }

        if let Some(bin_file) = detect_and_build(&temp_dir, &package_name) {
            let final_dest = install_dir.join(&package_name);
            
            println!("📥 Installing binary...");
            copy_with_sudo(&bin_file, &final_dest);

            config.packages.insert(package_name.clone(), PackageEntry {
                source: url.to_string(),
                tags: tags.to_vec(),
                generation: 1,
                bin_path: final_dest,
            });
            save_config(&config);
            println!("✅ Successfully installed '{}'!", package_name);
        }
    }

    pub fn remove(name: &str) {
        let mut config = load_config();
        if let Some(entry) = config.packages.remove(name) {
            println!("🗑️ Removing binary: {}", entry.bin_path.display());
            if entry.bin_path.exists() {
                // Try normal removal, fall back to sudo if it fails
                if fs::remove_file(&entry.bin_path).is_err() {
                    let _ = Command::new("sudo").arg("rm").arg(&entry.bin_path).status();
                }
            }
            save_config(&config);
            println!("✅ Removed '{}' from flix.", name);
        } else {
            println!("⚠️ Package '{}' not found.", name);
        }
    }

    pub fn update(name: Option<&str>) {
        if let Some(pkg_name) = name {
            println!("🔄 Updating '{}'...", pkg_name);
            // Logic: find source, re-clone, re-build
        } else {
            println!("🔄 Updating all packages...");
        }
    }

    pub fn list(tag: Option<&str>) {
        let config = load_config();
        if config.packages.is_empty() {
            println!("📭 No packages installed.");
            return;
        }

        println!("📋 Installed packages:");
        for (name, entry) in config.packages.iter() {
            if let Some(t) = tag {
                if !entry.tags.contains(&t.to_string()) { continue; }
            }
            println!("  📦 {} -> {}", name, entry.bin_path.display());
        }
    }

    fn ensure_dir_exists(path: &Path) {
        if !path.exists() {
            let status = Command::new("mkdir").arg("-p").arg(path).status();
            if status.is_err() || !status.unwrap().success() {
                println!("🔐 Permissions required to create {}. Trying sudo...", path.display());
                Command::new("sudo").arg("mkdir").arg("-p").arg(path).status().expect("Sudo failed");
            }
        }
    }

    fn copy_with_sudo(from: &Path, to: &Path) {
        let result = fs::copy(from, to);
        if result.is_err() {
            println!("🔐 Permissions required to write to {}. Trying sudo...", to.display());
            Command::new("sudo")
                .arg("cp")
                .arg(from)
                .arg(to)
                .status()
                .expect("Failed to copy even with sudo");
        }
    }

    fn detect_and_build(path: &Path, name: &str) -> Option<PathBuf> {
        if path.join("Cargo.toml").exists() {
            println!("🦀 Rust project detected. Building...");
            let _ = Command::new("cargo").arg("build").arg("--release").current_dir(path).status();
            let bin = path.join("target/release").join(name);
            if bin.exists() { return Some(bin); }
        }
        None
    }

    pub fn set_default_path(new_path: &str) {
        let mut config = load_config();
        let mut p = PathBuf::from(new_path);
        if !p.ends_with("flix") { p = p.join("flix"); }
        
        ensure_dir_exists(&p);
        config.default_install_path = Some(p.clone());
        save_config(&config);
        println!("⚙️ Global default path updated to: {}", p.display());
    }
}
