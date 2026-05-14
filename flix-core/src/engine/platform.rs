use std::env;

pub struct SystemInfo {
    pub os: String,
    pub arch: String,
}

pub fn get_system_info() -> SystemInfo {
    SystemInfo {
        os: env::consts::OS.to_string(),     // "linux", "macos", "windows"
        arch: env::consts::ARCH.to_string(), // "aarch64", "x86_64"
    }
}

/// Generates common naming patterns for binaries based on system info
pub fn get_binary_patterns(package_name: &str) -> Vec<String> {
    let info = get_system_info();
    let mut patterns = Vec::new();

    // Standard patterns: "name-linux-aarch64", "name-aarch64-unknown-linux-gnu", etc.
    patterns.push(format!("{}-{}-{}", package_name, info.os, info.arch));
    patterns.push(format!("{}-{}", package_name, info.arch));
    
    // Distro-specific (for later expansion)
    if info.os == "linux" {
        patterns.push("linux".to_string());
        patterns.push("musl".to_string());
    }

    patterns
}
