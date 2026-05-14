use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct FlixConfig {
    pub default_install_path: Option<PathBuf>,
    pub packages: BTreeMap<String, PackageEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PackageEntry {
    pub source: String,
    pub tags: Vec<String>,
    pub version_hash: String,
    pub version_tag: Option<String>,
    pub bin_path: PathBuf,
}
