use crate::config::load_config;
use crate::flags::SharedArgs;

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
