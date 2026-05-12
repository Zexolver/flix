pub mod config {
    use directories::ProjectDirs;
    use serde::{Deserialize, Serialize};
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::PathBuf;

    // BTreeMap keeps our packages perfectly alphabetized automatically!
    #[derive(Debug, Serialize, Deserialize, Default)]
    pub struct FlixConfig {
        pub packages: BTreeMap<String, PackageEntry>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct PackageEntry {
        pub source: String,
        pub tags: Vec<String>,
        pub generation: u32,
    }

    /// Determines where the config file should live (~/.config/flix/config.toml)
    pub fn get_config_path() -> PathBuf {
        if let Some(proj_dirs) = ProjectDirs::from("", "", "flix") {
            proj_dirs.config_dir().join("config.toml")
        } else {
            // Fallback if home directory is somehow missing
            PathBuf::from(".flix_config.toml")
        }
    }

    /// Reads the TOML file from disk. If it doesn't exist, returns an empty config.
    pub fn load_config() -> FlixConfig {
        let path = get_config_path();
        if path.exists() {
            let contents = fs::read_to_string(path).unwrap_or_default();
            toml::from_str(&contents).unwrap_or_default()
        } else {
            FlixConfig::default()
        }
    }

    /// Saves the config structure back to the TOML file.
    pub fn save_config(config: &FlixConfig) {
        let path = get_config_path();
        
        // Ensure the ~/.config/flix directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("Failed to create config directory");
        }

        let toml_string = toml::to_string(config).expect("Failed to serialize config");
        fs::write(path, toml_string).expect("Failed to write config file");
    }
}

pub mod engine {
    use super::config::{load_config, save_config, PackageEntry};

    pub fn install(url: &str, default: bool, quiet: bool, yes: bool, tags: &[String], path: Option<&str>) {
        println!("🚀 Preparing to install: {}", url);
        
        // --- 1. Here is where the actual git clone/build logic will go ---
        // For now, we simulate a successful build.
        println!("🔨 Simulating build process...");

        // --- 2. Extract a fake "name" from the URL for the config file ---
        // e.g., turn "https://github.com/alacritty/alacritty" into "alacritty"
        let package_name = url.split('/').last().unwrap_or("unknown_app").to_string();

        // --- 3. Save it to our state file ---
        let mut config = load_config();
        
        let entry = PackageEntry {
            source: url.to_string(),
            tags: tags.to_vec(),
            generation: 1, // First install = Gen 1
        };

        config.packages.insert(package_name.clone(), entry);
        save_config(&config);

        println!("✅ Installed '{}' and saved to config!", package_name);
    }

    pub fn remove(name: &str) {
        let mut config = load_config();
        
        if config.packages.remove(name).is_some() {
            save_config(&config);
            println!("🗑️  Removed package: {} from config.", name);
        } else {
            println!("⚠️  Package '{}' not found in config.", name);
        }
    }

    pub fn update(name: Option<&str>) {
        println!("🔄 Update logic not yet implemented.");
    }

    pub fn list(tag: Option<&str>) {
        let config = load_config();
        
        if config.packages.is_empty() {
            println!("📋 No packages installed yet.");
            return;
        }

        println!("📋 Installed Packages:");
        for (name, entry) in config.packages.iter() {
            // If a tag filter was provided, skip packages that don't have it
            if let Some(t) = tag {
                if !entry.tags.contains(&t.to_string()) {
                    continue;
                }
            }
            
            println!("  📦 {} (Gen {})", name, entry.generation);
            println!("     🔗 {}", entry.source);
            if !entry.tags.is_empty() {
                println!("     🏷️  {:?}", entry.tags);
            }
            println!();
        }
    }
}
