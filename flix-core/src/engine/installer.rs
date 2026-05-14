use crate::config::{load_config, save_config, interactive_setup, PackageEntry};
use crate::flags::SharedArgs;
use crate::engine::{system, git_manager, builder};
use std::env;
use std::fs;
use std::path::PathBuf;

pub fn install(
    url: &str, 
    shared: SharedArgs, 
    _use_release: bool, // Reserved for the agnostic release scraper
    _is_default: bool, 
    git_ref: Option<String>
) {
    let mut config = load_config();
    let url_clean = url.trim_end_matches(".git");
    let package_name = url_clean.split('/').last().unwrap_or("app").to_string();

    // --- 1. Handle Tag Merging and Overwrite Checks ---
    let is_installed = config.packages.contains_key(&package_name);
    
    if is_installed && !shared.force {
        eprintln!("❌ Error: Package '{}' already installed. Use -f to overwrite.", package_name);
        return;
    }

    let mut final_tags = if is_installed {
        // Merge: Start with existing tags, add new ones from -t if not already present
        let mut t = config.packages.get(&package_name).unwrap().tags.clone();
        for new_tag in &shared.tags {
            if !t.contains(new_tag) {
                t.push(new_tag.clone());
            }
        }
        t
    } else {
        shared.tags.clone()
    };

    // Auto-tag based on provider if not already tagged
    if url.contains("github.com") && !final_tags.contains(&"github".to_string()) { 
        final_tags.push("github".into()); 
    } else if url.contains("codeberg.org") && !final_tags.contains(&"codeberg".to_string()) {
        final_tags.push("codeberg".into());
    }

    // --- 2. Determine Installation Path ---
    let bin_dir = if let Some(p) = &shared.path {
        PathBuf::from(p)
    } else if let Some(p) = config.default_install_path.clone() {
        p
    } else {
        let chosen = interactive_setup();
        config.default_install_path = Some(chosen.clone());
        save_config(&config);
        chosen
    };

    // --- 3. Fetch and Build ---
    println!("🚀 Installing '{}'...", package_name);
    let temp_dir = env::temp_dir().join("flix_builds").join(&package_name);
    if temp_dir.exists() { let _ = fs::remove_dir_all(&temp_dir); }

    let commit_hash = match git_manager::fetch_and_checkout(url, &temp_dir, git_ref.clone()) {
        Ok(hash) => hash,
        Err(e) => { eprintln!("❌ {}", e); return; }
    };

    if let Some(bin_file) = builder::detect_and_build(&temp_dir, &package_name, shared.quiet) {
        system::ensure_dir_exists(&bin_dir);
        let final_dest = bin_dir.join(&package_name);
        system::copy_with_sudo(&bin_file, &final_dest);

        // --- 4. Update Registry ---
        config.packages.insert(package_name.clone(), PackageEntry {
            source: url.to_string(),
            tags: final_tags,
            version_hash: commit_hash[..8].to_string(),
            version_tag: git_ref, // Saves the tag/branch name used (e.g., v0.1.2)
            bin_path: final_dest,
        });
        
        save_config(&config);
        println!("✅ Installed '{}'!", package_name);
    } else {
        eprintln!("❌ Build failed or binary could not be located.");
    }
}
