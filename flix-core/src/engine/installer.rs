use crate::config::{load_config, save_config, PackageEntry, FlixConfig};
use crate::flags::SharedArgs;
use crate::engine::{system, git_manager, builder, platform};
use std::{env, fs};
use std::io::Read;
use std::path::PathBuf;
use flate2::read::GzDecoder;
use tar::Archive;

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

    if config.packages.contains_key(&package_name) && !shared.force {
        eprintln!("❌ Error: Package '{}' already installed. Use -f to overwrite.", package_name);
        return;
    }

    if use_release {
        println!("🔍 Searching for pre-built binary for '{}'...", package_name);
        if let Some(bin_path) = try_download_release(url_clean, &package_name, git_ref.as_deref()) {
            finalize_install(&mut config, &package_name, url, bin_path, &shared, "RELEASE", git_ref);
            return;
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

fn try_download_release(base_url: &str, name: &str, tag: Option<&str>) -> Option<PathBuf> {
    let terms = platform::get_search_terms();
    let os_term = platform::get_system_info().os.to_lowercase();
    let agent = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

    let release_page_url = if let Some(t) = tag {
        format!("{}/releases/tag/{}", base_url, t)
    } else {
        format!("{}/releases/latest", base_url)
    };

    let res = ureq::get(&release_page_url).set("User-Agent", agent).call().ok()?;
    let final_url = res.get_url().to_string();
    let mut html = res.into_string().ok()?;
    
    // Step 1: Check if we have links. If not, try the "expanded_assets" endpoint
    let mut download_link = scan_html_for_link(&html, &terms, &os_term);

    if download_link.is_none() {
        // Try to find the tag from the final redirected URL to build the expanded_assets URL
        let detected_tag = final_url.split('/').last().unwrap_or("");
        if !detected_tag.is_empty() {
            let expanded_url = format!("{}/releases/expanded_assets/{}", base_url, detected_tag);
            if let Ok(res) = ureq::get(&expanded_url).set("User-Agent", agent).call() {
                if let Ok(extra_html) = res.into_string() {
                    html = extra_html;
                    download_link = scan_html_for_link(&html, &terms, &os_term);
                }
            }
        }
    }

    let dl_path = download_link?;
    let full_dl_url = if dl_path.starts_with("http") {
        dl_path
    } else {
        let domain = base_url.split('/').take(3).collect::<Vec<_>>().join("/");
        format!("{}{}{}", domain, if dl_path.starts_with('/') { "" } else { "/" }, dl_path)
    };

    println!("📥 Found match: {}", full_dl_url.split('/').last().unwrap_or("binary"));
    
    let response = ureq::get(&full_dl_url).set("User-Agent", agent).call().ok()?;
    let mut buffer = Vec::new();
    response.into_reader().read_to_end(&mut buffer).ok()?;

    let temp_dir = env::temp_dir().join("flix_dl").join(name);
    let _ = fs::create_dir_all(&temp_dir);
    let target_path = temp_dir.join(name);

    // Extraction logic
    if full_dl_url.ends_with(".tar.gz") || full_dl_url.ends_with(".tgz") {
        let tar = GzDecoder::new(&buffer[..]);
        let mut archive = Archive::new(tar);
        for entry in archive.entries().ok()? {
            let mut file = entry.ok()?;
            if file.header().entry_type().is_file() {
                let p = file.path().ok()?;
                let fname = p.file_name()?.to_str()?;
                if fname.contains(name) || fname == name {
                    file.unpack(&target_path).ok()?;
                    return Some(target_path);
                }
            }
        }
    } else {
        fs::write(&target_path, buffer).ok()?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&target_path, fs::Permissions::from_mode(0o755)).ok()?;
        }
    }

    Some(target_path)
}

fn scan_html_for_link(html: &str, terms: &[String], os_term: &str) -> Option<String> {
    for part in html.split(|c| c == '"' || c == '\'' || c == '>') {
        let part_clean = part.split('<').next().unwrap_or("").trim();
        if part_clean.contains("/releases/download/") {
            let part_lower = part_clean.to_lowercase();
            let has_os = part_lower.contains(os_term);
            let has_arch = terms.iter().skip(1).any(|t| part_lower.contains(t));

            if has_os && has_arch {
                if [".sha256", ".asc", ".sig", ".sha256sum", ".sha1"].iter().any(|ext| part_lower.ends_with(ext)) {
                    continue;
                }
                return Some(part_clean.to_string());
            }
        }
    }
    None
}

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

    let mut final_tags = if let Some(existing) = config.packages.get(name) {
        let mut t = existing.tags.clone();
        for nt in &shared.tags { if !t.contains(nt) { t.push(nt.clone()); } }
        t
    } else {
        shared.tags.clone()
    };

    if url.contains("github.com") && !final_tags.contains(&"github".into()) { final_tags.push("github".into()); }

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
