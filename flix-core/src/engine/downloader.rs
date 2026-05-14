//! Handles downloading, error safety checks, and smart archive extraction of remote assets.

use std::{env, fs};
use std::io::Read;
use std::path::PathBuf;
use flate2::read::GzDecoder;
use tar::Archive;

const USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

/// Downloads a binary or archive from a URL and extracts it to a temporary directory.
/// Returns `Some(PathBuf)` containing the executable path if successful, or `None` if asset matching fails.
pub fn download_and_unpack(url: &str, package_name: &str) -> Option<PathBuf> {
    println!("📥 Fetching: {}", url.split('/').last().unwrap_or("binary"));
    
    // Fetch raw asset bytes over HTTP
    let response = ureq::get(url).set("User-Agent", USER_AGENT).call().ok()?;
    let mut buffer = Vec::new();
    response.into_reader().read_to_end(&mut buffer).ok()?;

    // Set up a dedicated temporary directory for extraction
    let temp_dir = env::temp_dir().join("flix_dl").join(package_name);
    let _ = fs::create_dir_all(&temp_dir);
    let target_path = temp_dir.join(package_name);

    // Handle compressed tarball archives (.tar.gz / .tgz)
    if url.ends_with(".tar.gz") || url.ends_with(".tgz") {
        let tar = GzDecoder::new(&buffer[..]);
        let mut archive = Archive::new(tar);
        
        for entry in archive.entries().ok()? {
            let mut file = entry.ok()?;
            if file.header().entry_type().is_file() {
                let p = file.path().ok()?;
                let fname = p.file_name()?.to_str()?;
                
                // Normalization variables to prevent naming mismatches (e.g., package 'pfetch-rs' containing binary 'pfetch')
                let clean_pkg = package_name.to_lowercase().replace("-rs", "");
                let fname_lower = fname.to_lowercase();
                
                // Smart Match Heuristics:
                // 1. Full string match
                // 2. Binary matches package with '-rs' stripped
                // 3. Package name contains the binary name (catches 'pfetch' inside 'pfetch-rs')
                if fname_lower == package_name.to_lowercase()
                    || fname_lower == clean_pkg
                    || package_name.to_lowercase().contains(&fname_lower)
                {
                    // Edge-case filter: Ignore supplemental documentation or licensing files matching the name
                    if [".md", ".txt", "license", "readme"].iter().any(|ext| fname_lower.contains(ext)) {
                        continue;
                    }
                    
                    // Unpack and return the verified path
                    file.unpack(&target_path).ok()?;
                    return Some(target_path);
                }
            }
        }
        
        // FIX: If we iterated through the entire archive without finding the binary, 
        // return None so the installer knows to execute its source-build fallback.
        None
    } else {
        // Handle raw standalone binary downloads
        fs::write(&target_path, buffer).ok()?;
        #[cfg(unix)]
        {
            // Ensure permissions are configured correctly for running a command-line program
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&target_path, fs::Permissions::from_mode(0o755)).ok()?;
        }
        Some(target_path)
    }
}
