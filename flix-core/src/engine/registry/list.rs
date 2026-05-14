use crate::config::load_config;
use crate::flags::SharedArgs;

pub fn list(shared: SharedArgs) {
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

        // Logic: Prioritize Tag (v0.1.0) -> Hash (a1b2c3d4)
        let version_display = entry.version_tag.as_deref().unwrap_or(&entry.version_hash);

        println!("{:<20} {:<15} {:<20?}", name, version_display, entry.tags);
    }
}
