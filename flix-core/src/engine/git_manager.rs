use git2::Repository;
use std::path::Path;

/// Clones a repo and optionally checks out a specific tag/commit. Returns the commit hash.
pub fn fetch_and_checkout(url: &str, temp_dir: &Path, git_ref: Option<String>) -> Result<String, String> {
    println!("📦 Cloning repository...");
    let repo = match Repository::clone(url, temp_dir) {
        Ok(r) => r,
        Err(e) => return Err(format!("Git Error: {}", e)),
    };

    // Handle -V / --git-ref
    if let Some(reference) = git_ref {
        println!("⚓ Checking out version: {}...", reference);
        match repo.revparse_ext(&reference) {
            Ok((object, _)) => {
                let _ = repo.checkout_tree(&object, None);
                let _ = repo.set_head_detached(object.id());
            }
            Err(_) => return Err(format!("Git Ref '{}' not found.", reference)),
        }
    }

    let head = repo.head().map_err(|_| "Failed to get HEAD".to_string())?;
    let commit_hash = head.target().unwrap().to_string();

    Ok(commit_hash)
}
