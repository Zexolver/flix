use crate::engine::downloader::extract::unpack_tarball;
use std::io::Read;
use std::path::PathBuf;
use std::{env, fs};

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

    // Route to extraction logic or direct dump
    if url.ends_with(".tar.gz") || url.ends_with(".tgz") {
        unpack_tarball(&buffer, package_name, &target_path)?;
        Some(target_path)
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
