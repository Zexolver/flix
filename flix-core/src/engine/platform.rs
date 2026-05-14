use std::env;

pub struct SystemInfo {
    pub os: String,
    pub arch: String,
}

pub fn get_system_info() -> SystemInfo {
    SystemInfo {
        os: env::consts::OS.to_string(),
        arch: env::consts::ARCH.to_string(),
    }
}

pub fn get_search_terms() -> Vec<String> {
    let info = get_system_info();
    let mut terms = Vec::new();
    
    // Add fuzzy matches for architecture
    let arch = info.arch.to_lowercase();
    terms.push(arch.clone());
    if arch == "aarch64" { terms.push("arm64".to_string()); }
    if arch == "x86_64" { terms.push("amd64".to_string()); }
    
    terms
}

pub fn get_binary_patterns(package_name: &str) -> Vec<String> {
    let info = get_system_info();
    let mut patterns = Vec::new();
    patterns.push(format!("{}-{}-{}", package_name, info.os, info.arch));
    patterns.push(format!("{}-{}", package_name, info.arch));
    if info.os == "linux" {
        patterns.push("linux".to_string());
        patterns.push("musl".to_string());
    }
    patterns
}
