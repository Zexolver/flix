use crate::config::{load_config, save_config};
use crate::flags::SharedArgs;
use crate::engine::installer::install;
use std::process::Command;
use std::io::{self, Write};

pub fn list(shared: SharedArgs, show_version: bool) {
    let config = load_config();
    println!("{:<20} {:<15} {:<20}", "Package", "Version", "Tags");
    println!("{:-<55}", "");
    
    for (name, entry) in config.packages.iter() {
        if !shared.tags.is_empty() {
            if !shared.tags.iter().any(|t| entry.tags.contains(t)) { continue; }
        }

        if let Some(ref p) = shared.path {
            if entry.bin_path.parent().map(|d| d.to_string_lossy().to_string()) != Some(p.clone()) {
                continue;
            }
        }

        // Logic: Prioritize Tag (v0.1.0) -> Hash (a1b2c3d4) -> Default (---)
        let version_display = if show_version {
            entry.version_tag.as_ref().unwrap_or(&entry.version_hash)
        } else {
            "---"
        };

        println!("{:<20} {:<15} {:<20?}", name, version_display, entry.tags);
    }
}

pub fn remove(name: &str, shared: SharedArgs) {
    let mut config = load_config();
    
    if !shared.yes {
        print!("Are you sure you want to remove '{}'? [y/N]: ", name);
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        if input.trim().to_lowercase() != "y" {
            println!("❌ Aborted.");
            return;
        }
    }

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

pub fn update(name: Option<&str>, shared: SharedArgs, release: bool) {
    let config = load_config();
    let mut targets = Vec::new();

    if let Some(pkg_name) = name {
        if let Some(entry) = config.packages.get(pkg_name) {
            targets.push((pkg_name.to_string(), entry.clone()));
        } else {
            println!("⚠️ Package '{}' not found.", pkg_name);
            return;
        }
    } else {
        if config.packages.is_empty() {
            println!("📋 No packages installed to update.");
            return;
        }
        for (pkg_name, entry) in config.packages.iter() {
            targets.push((pkg_name.clone(), entry.clone()));
        }
    }

    for (pkg_name, entry) in targets {
        if !shared.force {
            println!("✅ '{}' is already up to date. Use -f to force a fresh install.", pkg_name);
        } else {
            println!("🔄 Force updating '{}'...", pkg_name);
            // Pass the existing version_tag through to the installer so we don't lose it
            install(&entry.source, shared.clone(), release, false, entry.version_tag);
        }
    }
}
