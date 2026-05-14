use crate::config::load_config;
use crate::flags::SharedArgs;
use crate::engine::installer::install;

pub fn update(name: Option<&str>, shared: SharedArgs, release: bool) {
    let config = load_config();
    let mut targets = Vec::new();

    if let Some(pkg_name) = name {
        if let Some(entry) = config.packages.get(pkg_name) {
            targets.push((pkg_name.to_string(), entry.clone()));
        } else {
            println!("⚠️ Package '{}' not found.", pkg_name);
            return;
        }
    } else {
        if config.packages.is_empty() {
            println!("📋 No packages installed to update.");
            return;
        }
        for (pkg_name, entry) in config.packages.iter() {
            targets.push((pkg_name.clone(), entry.clone()));
        }
    }

    for (pkg_name, entry) in targets {
        if !shared.force {
            println!("✅ '{}' is already up to date. Use -f to force a fresh install.", pkg_name);
        } else {
            println!("🔄 Force updating '{}'...", pkg_name);
            // Pass the existing version_tag through to the installer so we don't lose it
            install(&entry.source, shared.clone(), release, false, entry.version_tag);
        }
    }
}
