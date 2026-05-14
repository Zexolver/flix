pub mod path;
pub mod completion;

use crate::config::load_config;
use crate::engine::system::fs::ensure_dir_exists;
use std::env;
use std::path::PathBuf;

pub fn shell_init() {
    let config = load_config();
    let base_dir = config.default_install_path.unwrap_or_else(|| PathBuf::from("/usr/local/flix/bin"));
    let etc_dir = base_dir.parent().unwrap().join("etc"); 
    
    ensure_dir_exists(&etc_dir);
    let home = env::var("HOME").unwrap_or_else(|_| "/home".into());
    let mut updated = false;

    // Delegate to our helper modules
    updated |= path::setup_path(&base_dir, &home);
    updated |= completion::setup_autocomplete(&etc_dir, &home);

    if updated {
        println!("\n✨ PATH and Autocomplete updated! To use immediately, run:");
        println!("    source ~/.bashrc");
    }
}
