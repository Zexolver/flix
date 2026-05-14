use crate::config::{load_config, save_config};

/// Manages tags for a given package. 
/// Returns a Result containing a tuple of (added_tags, removed_tags) on success.
pub fn manage_tags(name: &str, add: Vec<String>, remove: Vec<String>) -> Result<(Vec<String>, Vec<String>), String> {
    let mut config = load_config();

    if let Some(entry) = config.packages.get_mut(name) {
        let mut added = Vec::new();
        let mut removed = Vec::new();

        for t in remove {
            // Find the index of the tag in the Vec and remove it
            if let Some(pos) = entry.tags.iter().position(|x| x == &t) {
                entry.tags.remove(pos);
                removed.push(t);
            }
        }

        for t in add {
            // Only add the tag if it doesn't already exist in the Vec
            if !entry.tags.contains(&t) {
                entry.tags.push(t.clone());
                added.push(t);
            }
        }

        if !added.is_empty() || !removed.is_empty() {
            save_config(&config);
        }

        Ok((added, removed))
    } else {
        Err(format!("Package '{}' is not installed.", name))
    }
}
