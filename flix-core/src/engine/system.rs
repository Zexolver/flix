use crate::config::load_config;
use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn ensure_dir_exists(path: &Path) {
    if !path.exists() {
        let _ = Command::new("sudo").arg("mkdir").arg("-p").arg(path).status();
    }
}

pub fn copy_with_sudo(from: &Path, to: &Path) {
    let _ = Command::new("sudo").arg("cp").arg(from).arg(to).status();
    let _ = Command::new("sudo").arg("chmod").arg("+x").arg(to).status();
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
