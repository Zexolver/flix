use crate::config::{load_config, save_config};
use crate::flags::SharedArgs;
use std::process::Command;
use std::io::{self, Write};

pub fn remove(name: &str, shared: SharedArgs) {
    let mut config = load_config();
    
    if !shared.yes {
        print!("Are you sure you want to remove '{}'? [y/N]: ", name);
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        if input.trim().to_lowercase() != "y" {
            println!("❌ Aborted.");
            return;
        }
    }

    if let Some(entry) = config.packages.remove(name) {
        if entry.bin_path.exists() {
            let _ = Command::new("sudo").arg("rm").arg(&entry.bin_path).status();
        }
        save_config(&config);
        println!("✅ Removed '{}'.", name);
    } else {
        println!("⚠️ Package '{}' not found.", name);
    }
}
