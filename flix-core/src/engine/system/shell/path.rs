use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

pub(crate) fn setup_path(base_dir: &Path, home: &str) -> bool {
    let path_str = base_dir.to_string_lossy().to_string();
    let path_line = format!("\n# Flix Package Manager\nexport PATH=\"$PATH:{}\"", path_str);
    
    let shells = [".bashrc", ".zshrc", ".profile"];
    let mut updated = false;

    for sh in shells {
        let p = PathBuf::from(home).join(sh);
        if p.exists() {
            let contents = fs::read_to_string(&p).unwrap_or_default();
            
            if !contents.contains(&path_str) {
                if let Ok(mut file) = OpenOptions::new().append(true).open(&p) {
                    if let Err(e) = writeln!(file, "{}", path_line) {
                        eprintln!("❌ Failed to write to {}: {}", sh, e);
                    } else {
                        println!("✅ Added Flix to {}", sh);
                        updated = true;
                    }
                }
            } else {
                println!("ℹ️ Flix path already exists in {}", sh);
            }
        }
    }
    updated
}
