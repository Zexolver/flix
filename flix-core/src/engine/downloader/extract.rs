use flate2::read::GzDecoder;
use std::path::Path;
use tar::Archive;

/// Attempts to parse a compressed tarball, extract the correct binary matching the heuristics,
/// and copy it to the target path.
pub(crate) fn unpack_tarball(buffer: &[u8], package_name: &str, target_path: &Path) -> Option<()> {
    let tar = GzDecoder::new(buffer);
    let mut archive = Archive::new(tar);
    
    for entry in archive.entries().ok()? {
        let mut file = entry.ok()?;
        if file.header().entry_type().is_file() {
            let p = file.path().ok()?;
            let fname = p.file_name()?.to_str()?;
            
            // Normalization variables to prevent naming mismatches
            let clean_pkg = package_name.to_lowercase().replace("-rs", "");
            let fname_lower = fname.to_lowercase();
            
            // Smart Match Heuristics
            if fname_lower == package_name.to_lowercase()
                || fname_lower == clean_pkg
                || package_name.to_lowercase().contains(&fname_lower)
            {
                // Edge-case filter: Ignore supplemental documentation or licensing files
                if [".md", ".txt", "license", "readme"].iter().any(|ext| fname_lower.contains(ext)) {
                    continue;
                }
                
                // Unpack and return success
                file.unpack(target_path).ok()?;
                return Some(());
            }
        }
    }
    
    // If we iterated through the entire archive without finding the binary
    None
}
