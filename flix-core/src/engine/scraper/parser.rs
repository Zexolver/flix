/// Helper function that parses raw HTML looking for hrefs that match our OS/Arch heuristics.
pub(crate) fn scan_html_for_link(html: &str, terms: &[String], os_term: &str) -> Option<String> {
    for part in html.split(|c| c == '"' || c == '\'' || c == '>') {
        let part_clean = part.split('<').next().unwrap_or("").trim();
        
        if part_clean.contains("/releases/download/") {
            let part_lower = part_clean.to_lowercase();
            let has_os = part_lower.contains(os_term);
            let has_arch = terms.iter().any(|t| !t.is_empty() && part_lower.contains(t));

            let is_arm_host = terms.iter().any(|t| t == "aarch64" || t == "arm64");
            let is_x86_host = terms.iter().any(|t| t == "x86_64" || t == "amd64");
            
            if is_arm_host && (part_lower.contains("x86_64") || part_lower.contains("amd64") || part_lower.contains("x64")) {
                continue;
            }
            if is_x86_host && (part_lower.contains("aarch64") || part_lower.contains("arm64") || part_lower.contains("arm")) {
                continue;
            }

            if has_os && has_arch {
                if [".sha256", ".asc", ".sig", ".sha256sum", ".sha1"].iter().any(|ext| part_lower.ends_with(ext)) {
                    continue;
                }
                return Some(part_clean.to_string());
            }
        }
    }
    None
}

/// Extracts the version tag directly out of a standard GitHub release asset download link.
pub fn extract_tag_from_url(url: &str) -> Option<String> {
    let parts: Vec<&str> = url.split("/releases/download/").collect();
    if parts.len() > 1 {
        return parts[1].split('/').next().map(|s| s.to_string());
    }
    None
}
