//! High-level orchestration for package installations.

use crate::config::{load_config, save_config, PackageEntry, FlixConfig};
use crate::flags::SharedArgs;
use crate::engine::{system, git_manager, builder, scraper, downloader};
use std::{env, fs};
use std::path::PathBuf;

/// The primary entry point for installing a package.
/// Attempts to find a pre-built release first, falling back to source compilation.
pub fn install(
    url: &str, 
    shared: SharedArgs, 
    use_release: bool, 
    _is_default: bool, 
    git_ref: Option<String>
) {
    let mut config = load_config();
    let url_clean = url.trim_end_matches(".git");
    let package_name = url_clean.split('/').last().unwrap_or("app").to_string();

    // Prevent duplicate installs unless forced
    if config.packages.contains_key(&package_name) && !shared.force {
        eprintln!("❌ Error: Package '{}' already installed. Use -f to overwrite.", package_name);
        return;
    }

    // Attempt Release Download
    if use_release {
        println!("🔍 Searching for pre-built binary for '{}'...", package_name);
        if let Some(dl_url) = scraper::find_github_asset_url(url_clean, git_ref.as_deref()) {
            if let Some(bin_path) = downloader::download_and_unpack(&dl_url, &package_name) {
                // Dynamically extract the version string instead of using a hardcoded "RELEASE" placeholder
                let version_tag = scraper::extract_tag_from_url(&dl_url)
                    .unwrap_or_else(|| "RELEASE".to_string());

                finalize_install(&mut config, &package_name, url, bin_path, &shared, &version_tag, git_ref);
                return;
            }
        }
        println!("⚠️ No matching binary found. Falling back to source build...");
    }

    // Fallback to Source Build
    println!("🚀 Building '{}' from source...", package_name);
    let temp_dir = env::temp_dir().join("flix_builds").join(&package_name);
    if temp_dir.exists() { let _ = fs::remove_dir_all(&temp_dir); }

    let commit_hash = match git_manager::fetch_and_checkout(url, &temp_dir, git_ref.clone()) {
        Ok(hash) => hash,
        Err(e) => { eprintln!("❌ {}", e); return; }
    };

    if let Some(bin_file) = builder::detect_and_build(&temp_dir, &package_name, shared.quiet) {
        finalize_install(&mut config, &package_name, url, bin_file, &shared, &commit_hash[..8], git_ref);
    } else {
        eprintln!("❌ Build failed or binary could not be located.");
    }
}

/// Finalizes the installation by moving the binary to the system path and updating the configuration.
fn finalize_install(
    config: &mut FlixConfig, 
    name: &str, 
    url: &str, 
    src_file: PathBuf, 
    shared: &SharedArgs, 
    hash: &str, 
    tag: Option<String>
) {
    let bin_dir = if let Some(p) = &shared.path {
        PathBuf::from(p)
    } else {
        config.default_install_path.clone().unwrap_or_else(|| PathBuf::from("/usr/local/flix/bin"))
    };

    system::ensure_dir_exists(&bin_dir);
    let final_dest = bin_dir.join(name);
    system::copy_with_sudo(&src_file, &final_dest);

    // Merge tags
    let mut final_tags = if let Some(existing) = config.packages.get(name) {
        let mut t = existing.tags.clone();
        for nt in &shared.tags { if !t.contains(nt) { t.push(nt.clone()); } }
        t
    } else {
        shared.tags.clone()
    };

    if url.contains("github.com") && !final_tags.contains(&"github".into()) { final_tags.push("github".into()); }

    // Register package in config
    config.packages.insert(name.to_string(), PackageEntry {
        source: url.to_string(),
        tags: final_tags,
        version_hash: hash.to_string(),
        version_tag: tag,
        bin_path: final_dest,
    });

    save_config(config);
    println!("✅ Installed '{}'!", name);
}
