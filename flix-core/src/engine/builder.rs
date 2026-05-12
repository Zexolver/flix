use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub fn detect_and_build(path: &Path, name: &str, quiet: bool) -> Option<PathBuf> {
    if path.join("Cargo.toml").exists() {
        println!("🛠️ Building '{}'...", name);
        let mut cmd = Command::new("cargo");
        cmd.arg("build").arg("--release").current_dir(path);

        if quiet {
            cmd.stdout(Stdio::null()).stderr(Stdio::null());
        }

        let status = cmd.status().ok()?;
        
        if status.success() {
            let bin = path.join("target/release").join(name);
            if bin.exists() { return Some(bin); }
            
            // Fallback: Find any executable in the release dir if names don't match exactly
            if let Ok(rd) = fs::read_dir(path.join("target/release")) {
                for entry in rd.flatten() {
                    let p = entry.path();
                    if p.is_file() && !p.extension().map_or(false, |e| e == "d") { 
                        return Some(p); 
                    }
                }
            }
        }
    }
    None
}
