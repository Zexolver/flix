use std::process::{Command, Stdio};
use std::path::{Path, PathBuf};
use std::fs;
use crate::flags::SharedArgs;
use crate::config::{Config, Package};
use git2::Repository;

/// The primary install logic
pub fn install(
    url: &str, 
    shared: SharedArgs, 
    use_release: bool, 
    is_default: bool, 
    git_ref: Option<String>
) {
    let name = url.split('/').last().unwrap_or("unknown-pkg").trim_end_matches(".git");
    let install_path = shared.path.unwrap_or_else(|| "/usr/local/bin".to_string());
    let final_dest = Path::new(&install_path).join(name);

    // 1. Handle --force
    if final_dest.exists() && !shared.force {
        println!("❌ Error: '{}' already exists at {}. Use -f to overwrite.", name, final_dest.display());
        return;
    }

    // 2. Decide: Download Binary or Build from Source?
    if use_release {
        println!("🔍 Searching for pre-built binaries for {}...", name);
        // TODO: Implement GitHub Release Scraper here
        println!("⚠️ Binary releases not yet implemented, falling back to source build.");
    }

    // 3. Build from Source
    let build_dir = Path::new("/tmp/flix_builds").join(name);
    if build_dir.exists() { fs::remove_dir_all(&build_dir).ok(); }
    fs::create_dir_all(&build_dir).ok();

    println!("git cloning...");
    let repo = match Repository::clone(url, &build_dir) {
        Ok(r) => r,
        Err(e) => { println!("❌ Clone failed: {}", e); return; }
    };

    // 4. Handle -V / --git-ref
    if let Some(reference) = git_ref {
        println!("⚓ Checking out version: {}...", reference);
        let (object, _) = repo.revparse_ext(&reference).expect("Ref not found");
        repo.checkout_tree(&object, None).expect("Checkout failed");
        repo.set_head_detached(object.id()).ok();
    }

    // 5. Build with -q / --quiet support
    println!("🛠️ Building {}...", name);
    let mut cmd = Command::new("cargo");
    cmd.arg("build").arg("--release").current_dir(&build_dir);

    if shared.quiet {
        cmd.stdout(Stdio::null()).stderr(Stdio::null());
    }

    let status = cmd.status().expect("Failed to run cargo build");

    if status.success() {
        let binary_path = build_dir.join("target/release").join(name);
        
        // 6. Install to final path (using sudo if needed)
        println!("🚚 Installing to {}...", final_dest.display());
        if let Err(e) = fs::copy(&binary_path, &final_dest) {
             println!("⚠️ Permission denied. Trying sudo...");
             Command::new("sudo")
                .arg("cp")
                .arg(&binary_path)
                .arg(&final_dest)
                .status()
                .ok();
        }

        // 7. Save metadata to config.toml
        let mut config = Config::load();
        config.packages.insert(name.to_string(), Package {
            url: url.to_string(),
            hash: "latest".to_string(), // In reality, get the actual commit hash here
            tags: shared.tags,
            is_default,
        });
        config.save();

        println!("✅ Successfully installed {}!", name);
    } else {
        println!("❌ Build failed.");
    }
}

/// The listing logic with -v (version) and -t (tag) support
pub fn list(shared: SharedArgs, show_version: bool) {
    let config = Config::load();
    println!("{:<15} {:<15} {:<20}", "Package", "Version", "Tags");
    println!("{}", "-".repeat(50));

    for (name, pkg) in config.packages {
        // Filter by tags if -t was provided
        if !shared.tags.is_empty() {
            if !shared.tags.iter().any(|t| pkg.tags.contains(t)) {
                continue;
            }
        }

        let version_display = if show_version { &pkg.hash } else { "---" };
        println!("{:<15} {:<15} {:?}", name, version_display, pkg.tags);
    }
}
