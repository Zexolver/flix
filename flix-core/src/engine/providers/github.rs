use crate::engine::platform;
use crate::engine::providers::traits::Provider;

const USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

pub struct GithubProvider;

impl Provider for GithubProvider {
    fn name(&self) -> &'static str {
        "GitHub"
    }

    fn supports(&self, url: &str) -> bool {
        url.contains("github.com")
    }

    fn find_asset_url(&self, repo_url: &str, git_ref: Option<&str>) -> Option<String> {
        let terms = platform::get_search_terms();
        let os_term = platform::get_system_info().os.to_lowercase();

        let release_page_url = if let Some(t) = git_ref {
            format!("{}/releases/tag/{}", repo_url, t)
        } else {
            format!("{}/releases/latest", repo_url)
        }
        .replace(".git", ""); // Ensure clean URLs just in case

        let res = ureq::get(&release_page_url).set("User-Agent", USER_AGENT).call().ok()?;
        let final_url = res.get_url().to_string();
        let mut html = res.into_string().ok()?;
        
        let mut download_link = scan_html_for_link(&html, &terms, &os_term);

        // Fallback for expanded assets if not on the main page
        if download_link.is_none() {
            let detected_tag = final_url.split('/').last().unwrap_or("");
            if !detected_tag.is_empty() {
                let expanded_url = format!("{}/releases/expanded_assets/{}", repo_url.replace(".git", ""), detected_tag);
                if let Ok(res) = ureq::get(&expanded_url).set("User-Agent", USER_AGENT).call() {
                    if let Ok(extra_html) = res.into_string() {
                        html = extra_html;
                        download_link = scan_html_for_link(&html, &terms, &os_term);
                    }
                }
            }
        }

        let dl_path = download_link?;
        
        let full_dl_url = if dl_path.starts_with("http") {
            dl_path
        } else {
            let domain = repo_url.split('/').take(3).collect::<Vec<_>>().join("/");
            format!("{}{}{}", domain, if dl_path.starts_with('/') { "" } else { "/" }, dl_path)
        };

        Some(full_dl_url)
    }

    fn extract_tag(&self, asset_url: &str) -> Option<String> {
        let parts: Vec<&str> = asset_url.split("/releases/download/").collect();
        if parts.len() > 1 {
            return parts[1].split('/').next().map(|s| s.to_string());
        }
        None
    }

    fn get_latest_tag(&self, repo_url: &str) -> Option<String> {
        let latest_url = format!("{}/releases/latest", repo_url.replace(".git", ""));
        
        // ureq will automatically follow the redirect from /latest to /tag/vX.X.X
        if let Ok(res) = ureq::get(&latest_url).set("User-Agent", USER_AGENT).call() {
            let final_url = res.get_url();
            let parts: Vec<&str> = final_url.split("/releases/tag/").collect();
            
            if parts.len() > 1 {
                return Some(parts[1].to_string());
            }
        }
        None
    }
}

/// Helper function that parses raw HTML looking for hrefs that match our OS/Arch heuristics.
fn scan_html_for_link(html: &str, terms: &[String], os_term: &str) -> Option<String> {
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
