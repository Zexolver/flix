pub mod builder;
pub mod git_manager;
pub mod installer;
pub mod registry;
pub mod system;
pub mod platform;
pub mod providers;
pub mod downloader;

// Re-export public functions so the CLI doesn't break
pub use installer::install;
pub use registry::{list, remove, update};
pub use system::{self_install, shell_init};
