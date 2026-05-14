use crate::config::load_config;
use crate::engine::system::fs::{copy_with_sudo, ensure_dir_exists};
use std::env;
use std::path::PathBuf;

pub fn self_install() {
    let config = load_config();
    let bin_dir = config.default_install_path.unwrap_or_else(|| PathBuf::from("/usr/local/flix/bin"));
    let current_exe = env::current_exe().expect("Failed to get current exe path");
    let target_path = bin_dir.join("flix");

    ensure_dir_exists(&bin_dir);
    copy_with_sudo(&current_exe, &target_path);
    println!("✅ Flix installed to {}. Run 'flix shell-init'.", target_path.display());
}
