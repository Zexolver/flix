use crate::config::{save_config, PackageEntry, FlixConfig};
use crate::flags::SharedArgs;
use crate::engine::system;
use std::path::PathBuf;

/// Finalizes the installation by moving the binary to the system path and updating the configuration.
pub(crate) fn finalize_install(
    config: &mut FlixConfig, 
    name: &str, 
    url: &str, 
    src_file: PathBuf, 
    shared: &SharedArgs, 
    hash: &str, 
    tag: Option<String>
) {
    let bin_dir = if let Some(p) = &shared.path {
        PathBuf::from(p)
    } else {
        config.default_install_path.clone().unwrap_or_else(|| PathBuf::from("/usr/local/flix/bin"))
    };

    system::ensure_dir_exists(&bin_dir);
    let final_dest = bin_dir.join(name);
    system::copy_with_sudo(&src_file, &final_dest);

    let mut final_tags = if let Some(existing) = config.packages.get(name) {
        let mut t = existing.tags.clone();
        for nt in &shared.tags { if !t.contains(nt) { t.push(nt.clone()); } }
        t
    } else {
        shared.tags.clone()
    };

    if url.contains("github.com") && !final_tags.contains(&"github".into()) { final_tags.push("github".into()); }

    config.packages.insert(name.to_string(), PackageEntry {
        source: url.to_string(),
        tags: final_tags,
        version_hash: hash.to_string(),
        version_tag: tag,
        bin_path: final_dest,
    });

    save_config(config);
    println!("✅ Installed '{}'!", name);
}
