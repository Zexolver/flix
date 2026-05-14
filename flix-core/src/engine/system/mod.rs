pub mod fs;
pub mod install;
pub mod shell;

// Re-export everything so the external API remains 100% identical
pub use fs::{copy_with_sudo, ensure_dir_exists};
pub use install::self_install;
pub use shell::shell_init;
