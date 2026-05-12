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

    /// Determines config path based on installation type.
    /// If /usr/local/flix exists and is writable (or we are sudo), use /usr/local/etc/flix
    /// Otherwise, fallback to user home config.
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
            if !parent.exists() {
                let _ = fs::create_dir_all(parent);
                // If standard create fails, we assume sudo is needed at the engine level,
                // but for config saving, we try to be direct.
            }
        }
        let toml_string = toml::to_string(config).expect("Failed to serialize config");
        if let Err(_) = fs::write(&path, toml_string) {
            // Fallback for system-wide config save if normal write fails
            let tmp_path = std::env::temp_dir().join("flix_config.tmp");
            let toml_string = toml::to_string(config).unwrap();
            fs::write(&tmp_path, toml_string).unwrap();
            let _ = std::process::Command::new("sudo")
                .arg("mkdir")
                .arg("-p")
                .arg(path.parent().unwrap())
                .status();
            let _ = std::process::Command::new("sudo")
                .arg("cp")
                .arg(&tmp_path)
                .arg(&path)
                .status();
        }
    }

    pub fn enforce_flix_dir(mut path: PathBuf) -> PathBuf {
        if !path.ends_with("flix") {
            path.push("flix");
        }
        path
    }

    pub fn interactive_setup() -> PathBuf {
        println!("\n👋 Welcome to Flix! Let's set up your default installation directory.");
        println!("All packages will be stored in a dedicated 'flix' subfolder.\n");
        
        println!("[1] System-wide: /usr/local/flix");
        
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

        match choice {
            "2" => enforce_flix_dir(PathBuf::from(user_default)),
            "3" => {
                print!("Enter custom path: ");
                io::stdout().flush().unwrap();
                let mut custom = String::new();
                io::stdin().read_line(&mut custom).unwrap();
                enforce_flix_dir(PathBuf::from(custom.trim()))
            }
            "0" => std::process::exit(0),
            _ => PathBuf::from("/usr/local/flix"),
        }
    }
}

pub mod engine {
    use super::config::{load_config, save_config, enforce_flix_dir, PackageEntry, interactive_setup};
    use git2::Repository;
    use std::env;
    use std::fs::{self, OpenOptions};
    use std::io::{Write, Read};
    use std::path::{Path, PathBuf};
    use std::process::Command;

    pub fn install(
        url: &str,
        force: bool,
        tags: &[String],
        custom_path: Option<&str>,
    ) {
        let mut config = load_config();

        let url_clean = url.trim_end_matches(".git");
        let package_name = url_clean.split('/').last().unwrap_or("app").to_string();

        if config.packages.contains_key(&package_name) && !force {
            eprintln!("❌ Error: Package '{}' is already installed. Use --force to overwrite.", package_name);
            return;
        }

        let install_dir = if let Some(p) = custom_path {
            enforce_flix_dir(PathBuf::from(p))
        } else if let Some(p) = config.default_install_path.clone() {
            p
        } else {
            let chosen = interactive_setup();
            config.default_install_path = Some(chosen.clone());
            save_config(&config);
            chosen
        };

        if !check_write_permission(&install_dir) {
            println!("🔐 Notice: {} requires elevated privileges. Sudo may be requested.", install_dir.display());
        }

        println!("🚀 Preparing to install '{}' to: {}", package_name, install_dir.display());

        let temp_dir = env::temp_dir().join("flix_builds").join(&package_name);
        if temp_dir.exists() { let _ = fs::remove_dir_all(&temp_dir); }

        println!("📦 Cloning repository...");
        if let Err(e) = Repository::clone(url, &temp_dir) {
            eprintln!("❌ Git Error: {}", e);
            return;
        }

        if let Some(bin_file) = detect_and_build(&temp_dir, &package_name) {
            ensure_dir_exists(&install_dir);
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
        } else {
            eprintln!("❌ Build failed: No Rust binary was generated.");
        }
    }

    pub fn update(name: Option<&str>, force: bool) {
        let config = load_config();
        
        if let Some(pkg_name) = name {
            if let Some(entry) = config.packages.get(pkg_name) {
                println!("🔄 Updating '{}' (Force: {})...", pkg_name, force);
                install(&entry.source, true, &entry.tags, None);
            } else {
                eprintln!("❌ Package '{}' not found.", pkg_name);
            }
        } else {
            if config.packages.is_empty() {
                println!("📭 Nothing to update.");
                return;
            }
            println!("🔄 Updating all packages...");
            let targets: Vec<(String, Vec<String>)> = config.packages.values()
                .map(|e| (e.source.clone(), e.tags.clone()))
                .collect();
            
            for (source, tags) in targets {
                install(&source, true, &tags, None);
            }
        }
    }

    pub fn shell_init() {
        let config = load_config();
        let Some(install_path) = config.default_install_path else {
            println!("⚠️ No default path set. Run an install first!");
            return;
        };

        let path_line = format!("\nexport PATH=\"$PATH:{}\"", install_path.display());
        let home = env::var("HOME").expect("Could not find HOME directory");
        
        let profiles = vec![
            format!("{}/.bashrc", home),
            format!("{}/.zshrc", home),
        ];

        let mut updated = false;
        for profile_path in profiles {
            let path = Path::new(&profile_path);
            if path.exists() {
                let mut contents = String::new();
                if let Ok(mut f) = fs::File::open(path) {
                    let _ = f.read_to_string(&mut contents);
                    if !contents.contains(&path_line.trim()) {
                        if let Ok(mut file) = OpenOptions::new().append(true).open(path) {
                            let _ = writeln!(file, "{}", path_line);
                            println!("✅ Added Flix to {}", profile_path);
                            updated = true;
                        }
                    } else {
                        println!("ℹ️ Flix is already in {}", profile_path);
                        updated = true;
                    }
                }
            }
        }

        if updated {
            println!("\n✨ Path updated! Restart your terminal or run: source ~/.bashrc");
        } else {
            println!("❌ No supported shell profile found. Manually add: {}", path_line.trim());
        }
    }

    pub fn remove(name: &str) {
        let mut config = load_config();
        if let Some(entry) = config.packages.remove(name) {
            println!("🗑️ Removing binary: {}", entry.bin_path.display());
            if entry.bin_path.exists() {
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

    fn check_write_permission(path: &Path) -> bool {
        if !path.exists() {
            return path.parent().map_or(false, |p| {
                fs::metadata(p).map(|m| !m.permissions().readonly()).unwrap_or(false)
            });
        }
        fs::metadata(path).map(|m| !m.permissions().readonly()).unwrap_or(false)
    }

    fn ensure_dir_exists(path: &Path) {
        if !path.exists() {
            if fs::create_dir_all(path).is_err() {
                let _ = Command::new("sudo").arg("mkdir").arg("-p").arg(path).status();
            }
        }
    }

    fn copy_with_sudo(from: &Path, to: &Path) {
        if fs::copy(from, to).is_err() {
            let _ = Command::new("sudo").arg("cp").arg(from).arg(to).status();
        }
        let _ = Command::new("sudo").arg("chmod").arg("+x").arg(to).status();
    }

    fn detect_and_build(path: &Path, name: &str) -> Option<PathBuf> {
        if path.join("Cargo.toml").exists() {
            println!("🦀 Rust project detected. Building...");
            let status = Command::new("cargo").arg("build").arg("--release").current_dir(path).status();
            if let Ok(s) = status {
                if s.success() {
                    // Try to find the binary. Some crates have different names than the repo.
                    // For now, we assume repo name, but check target/release/
                    let bin = path.join("target/release").join(name);
                    if bin.exists() { return Some(bin); }
                    
                    // Fallback: look for ANY executable in target/release if name doesn't match
                    if let Ok(entries) = fs::read_dir(path.join("target/release")) {
                        for entry in entries.flatten() {
                            let p = entry.path();
                            if p.is_file() && !p.extension().map_or(false, |ext| ext == "d" || ext == "rlib") {
                                // Basic heuristic for "is this the binary?"
                                return Some(p);
                            }
                        }
                    }
                }
            }
        }
        None
    }

    pub fn set_default_path(new_path: &str) {
        let mut config = load_config();
        let p = enforce_flix_dir(PathBuf::from(new_path));
        ensure_dir_exists(&p);
        config.default_install_path = Some(p.clone());
        save_config(&config);
        println!("⚙️ Global default path updated to: {}", p.display());
    }
}
