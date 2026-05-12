use crate::config::{load_config, save_config, PackageEntry, interactive_setup};
use git2::Repository;
use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn install(url: &str, force: bool, tags: &[String], custom_path: Option<&str>) {
    let mut config = load_config();
    let url_clean = url.trim_end_matches(".git");
    let package_name = url_clean.split('/').last().unwrap_or("app").to_string();

    if config.packages.contains_key(&package_name) && !force {
        eprintln!("❌ Error: Package '{}' already installed.", package_name);
        return;
    }

    let bin_dir = if let Some(p) = custom_path {
        PathBuf::from(p)
    } else if let Some(p) = config.default_install_path.clone() {
        p
    } else {
        let chosen = interactive_setup();
        config.default_install_path = Some(chosen.clone());
        save_config(&config);
        chosen
    };

    println!("🚀 Installing '{}'...", package_name);
    let temp_dir = env::temp_dir().join("flix_builds").join(&package_name);
    if temp_dir.exists() { let _ = fs::remove_dir_all(&temp_dir); }

    let repo = match Repository::clone(url, &temp_dir) {
        Ok(r) => r,
        Err(e) => { eprintln!("❌ Git Error: {}", e); return; }
    };

    let head = repo.head().unwrap();
    let commit_hash = head.target().unwrap().to_string();

    if let Some(bin_file) = detect_and_build(&temp_dir, &package_name) {
        ensure_dir_exists(&bin_dir);
        let final_dest = bin_dir.join(&package_name);
        copy_with_sudo(&bin_file, &final_dest);

        let mut final_tags = tags.to_vec();
        if url.contains("github.com") { final_tags.push("github".into()); }

        config.packages.insert(package_name.clone(), PackageEntry {
            source: url.to_string(),
            tags: final_tags,
            version_hash: commit_hash[..8].to_string(),
            bin_path: final_dest,
        });
        save_config(&config);
        println!("✅ Installed '{}'!", package_name);
    }
}

pub fn remove(name: &str) {
    let mut config = load_config();
    if let Some(entry) = config.packages.remove(name) {
        if entry.bin_path.exists() {
            let _ = Command::new("sudo").arg("rm").arg(&entry.bin_path).status();
        }
        save_config(&config);
        println!("✅ Removed '{}'.", name);
    } else {
        println!("⚠️ Package '{}' not found.", name);
    }
}

pub fn update(name: Option<&str>, _force: bool) {
    let config = load_config();
    if let Some(pkg_name) = name {
        if let Some(entry) = config.packages.get(pkg_name) {
            install(&entry.source, true, &entry.tags, None);
        }
    } else {
        for entry in config.packages.values() {
            install(&entry.source, true, &entry.tags, None);
        }
    }
}

pub fn list(tag: Option<&str>) {
    let config = load_config();
    println!("{:<15} {:<10} {:<20}", "Package", "Hash", "Tags");
    println!("{:-<45}", "");
    for (name, entry) in config.packages.iter() {
        if let Some(t) = tag {
            if !entry.tags.contains(&t.to_string()) { continue; }
        }
        println!("{:<15} {:<10} {:<20?}", name, entry.version_hash, entry.tags);
    }
}

pub fn self_install() {
    let config = load_config();
    let bin_dir = config.default_install_path.unwrap_or_else(|| PathBuf::from("/usr/local/flix/bin"));
    let current_exe = env::current_exe().expect("Failed to get current exe path");
    let target_path = bin_dir.join("flix");

    ensure_dir_exists(&bin_dir);
    copy_with_sudo(&current_exe, &target_path);
    println!("✅ Flix installed to {}. Run 'flix shell-init'.", target_path.display());
}

pub fn shell_init() {
    let config = load_config();
    if let Some(path) = config.default_install_path {
        let path_str = path.to_string_lossy().to_string();
        let line = format!("\n# Flix Package Manager\nexport PATH=\"$PATH:{}\"", path_str);
        
        let home = env::var("HOME").unwrap_or_else(|_| "/home".into());
        let shells = [".bashrc", ".zshrc", ".profile"];
        
        let mut updated = false;

        for sh in shells {
            let p = PathBuf::from(&home).join(sh);
            if p.exists() {
                let contents = fs::read_to_string(&p).unwrap_or_default();
                
                // Check if the path is already in the file to avoid bloat
                if !contents.contains(&path_str) {
                    if let Ok(mut file) = OpenOptions::new().append(true).open(&p) {
                        if let Err(e) = writeln!(file, "{}", line) {
                            eprintln!("❌ Failed to write to {}: {}", sh, e);
                        } else {
                            println!("✅ Added Flix to {}", sh);
                            updated = true;
                        }
                    }
                } else {
                    println!("ℹ️ Flix path already exists in {}", sh);
                    updated = true;
                }
            }
        }

        if updated {
            println!("\n✨ PATH updated! To use 'flix' immediately, run:");
            println!("   source ~/.bashrc  (or your shell's config file)");
        }
    } else {
        println!("⚠️ No default install path found in config. Run 'flix install' or 'flix setup' first.");
    }
}
fn ensure_dir_exists(path: &Path) {
    if !path.exists() {
        let _ = Command::new("sudo").arg("mkdir").arg("-p").arg(path).status();
    }
}

fn copy_with_sudo(from: &Path, to: &Path) {
    let _ = Command::new("sudo").arg("cp").arg(from).arg(to).status();
    let _ = Command::new("sudo").arg("chmod").arg("+x").arg(to).status();
}

fn detect_and_build(path: &Path, name: &str) -> Option<PathBuf> {
    if path.join("Cargo.toml").exists() {
        let status = Command::new("cargo").arg("build").arg("--release").current_dir(path).status();
        if status.ok()?.success() {
            let bin = path.join("target/release").join(name);
            if bin.exists() { return Some(bin); }
            if let Ok(rd) = fs::read_dir(path.join("target/release")) {
                for entry in rd.flatten() {
                    let p = entry.path();
                    if p.is_file() && !p.extension().map_or(false, |e| e == "d") { return Some(p); }
                }
            }
        }
    }
    None
}
