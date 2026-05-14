pub mod models;
pub mod io;

pub use io::{get_config_paths, interactive_setup, load_config, save_config};
pub use models::{FlixConfig, PackageEntry};
