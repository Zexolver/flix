use crate::config::load_config;
use crate::flags::SharedArgs;
use crate::engine::{git_manager, builder, scraper, downloader};
use crate::engine::installer::finalize::finalize_install;
use std::{env, fs};

/// The primary entry point for installing a package.
pub fn install(
    url: &str, 
    shared: SharedArgs, 
    use_release: bool, 
    _is_default: bool, 
    git_ref: Option<String>
) {
    // --- URL Guardrail ---
    if !url.starts_with("http://") && !url.starts_with("https://") && !url.starts_with("git://") && !url.starts_with("git@") {
        eprintln!("❌ Error: '{}' does not look like a valid repository URL.", url);
        println!("💡 Tip: If you are trying to update an existing package, run: flix update {}", url);
        println!("💡 Tip: If you are trying to install a new package, provide the full URL (e.g., https://github.com/user/repo)");
        std::process::exit(1);
    }
    // ---------------------

    let mut config = load_config();
    let url_clean = url.trim_end_matches(".git");
    let package_name = url_clean.split('/').last().unwrap_or("app").to_string();

    if config.packages.contains_key(&package_name) && !shared.force {
        eprintln!("❌ Error: Package '{}' already installed. Use -f to overwrite.", package_name);
        return;
    }

    if use_release {
        println!("🔍 Searching for pre-built binary for '{}'...", package_name);
        if let Some(dl_url) = scraper::find_github_asset_url(url_clean, git_ref.as_deref()) {
            if let Some(bin_path) = downloader::download_and_unpack(&dl_url, &package_name) {
                let version_tag = scraper::extract_tag_from_url(&dl_url)
                    .unwrap_or_else(|| "RELEASE".to_string());

                finalize_install(&mut config, &package_name, url, bin_path, &shared, &version_tag, git_ref);
                return;
            }
        }
        println!("⚠️ No matching binary found. Falling back to source build...");
    }

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
