//! Handles downloading and unpacking of remote assets.

use std::{env, fs};
use std::io::Read;
use std::path::PathBuf;
use flate2::read::GzDecoder;
use tar::Archive;

const USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

/// Downloads a binary or archive from a URL and extracts it to a temporary directory.
/// Returns the path to the extracted executable binary.
pub fn download_and_unpack(url: &str, package_name: &str) -> Option<PathBuf> {
    println!("📥 Fetching: {}", url.split('/').last().unwrap_or("binary"));
    
    // Download the raw bytes
    let response = ureq::get(url).set("User-Agent", USER_AGENT).call().ok()?;
    let mut buffer = Vec::new();
    response.into_reader().read_to_end(&mut buffer).ok()?;

    // Set up temp directory
    let temp_dir = env::temp_dir().join("flix_dl").join(package_name);
    let _ = fs::create_dir_all(&temp_dir);
    let target_path = temp_dir.join(package_name);

    // If it's an archive, unpack it and find the binary inside
    if url.ends_with(".tar.gz") || url.ends_with(".tgz") {
        let tar = GzDecoder::new(&buffer[..]);
        let mut archive = Archive::new(tar);
        
        for entry in archive.entries().ok()? {
            let mut file = entry.ok()?;
            if file.header().entry_type().is_file() {
                let p = file.path().ok()?;
                let fname = p.file_name()?.to_str()?;
                
                // Heuristic: If the file inside the archive shares a name with the package, it's the binary
                if fname.contains(package_name) || fname == package_name {
                    file.unpack(&target_path).ok()?;
                    return Some(target_path);
                }
            }
        }
    } else {
        // If it's a raw binary, just write it and make it executable
        fs::write(&target_path, buffer).ok()?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&target_path, fs::Permissions::from_mode(0o755)).ok()?;
        }
    }

    Some(target_path)
}
