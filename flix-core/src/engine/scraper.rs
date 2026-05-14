//! Handles web scraping logic to find release assets.

use crate::engine::platform;

const USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

/// Scrapes a GitHub repository's release page to find the appropriate download URL
/// for the current operating system and architecture.
pub fn find_github_asset_url(base_url: &str, tag: Option<&str>) -> Option<String> {
    let terms = platform::get_search_terms();
    let os_term = platform::get_system_info().os.to_lowercase();

    let release_page_url = if let Some(t) = tag {
        format!("{}/releases/tag/{}", base_url, t)
    } else {
        format!("{}/releases/latest", base_url)
    };

    // Attempt to fetch the primary release page
    let res = ureq::get(&release_page_url).set("User-Agent", USER_AGENT).call().ok()?;
    let final_url = res.get_url().to_string();
    let mut html = res.into_string().ok()?;
    
    // Step 1: Check if we have links in the initial HTML
    let mut download_link = scan_html_for_link(&html, &terms, &os_term);

    // Step 2: Fallback to GitHub's 'expanded_assets' hidden endpoint if the table was empty
    if download_link.is_none() {
        let detected_tag = final_url.split('/').last().unwrap_or("");
        if !detected_tag.is_empty() {
            let expanded_url = format!("{}/releases/expanded_assets/{}", base_url, detected_tag);
            if let Ok(res) = ureq::get(&expanded_url).set("User-Agent", USER_AGENT).call() {
                if let Ok(extra_html) = res.into_string() {
                    html = extra_html;
                    download_link = scan_html_for_link(&html, &terms, &os_term);
                }
            }
        }
    }

    let dl_path = download_link?;
    
    // Step 3: Format the final absolute URL
    let full_dl_url = if dl_path.starts_with("http") {
        dl_path
    } else {
        let domain = base_url.split('/').take(3).collect::<Vec<_>>().join("/");
        format!("{}{}{}", domain, if dl_path.starts_with('/') { "" } else { "/" }, dl_path)
    };

    Some(full_dl_url)
}

/// Helper function that parses raw HTML looking for hrefs that match our OS/Arch heuristics.
fn scan_html_for_link(html: &str, terms: &[String], os_term: &str) -> Option<String> {
    for part in html.split(|c| c == '"' || c == '\'' || c == '>') {
        let part_clean = part.split('<').next().unwrap_or("").trim();
        
        // Ensure it's actually a download link
        if part_clean.contains("/releases/download/") {
            let part_lower = part_clean.to_lowercase();
            let has_os = part_lower.contains(os_term);
            let has_arch = terms.iter().skip(1).any(|t| part_lower.contains(t));

            // Must match OS and Arch, but strictly ignore checksum/signature files
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
