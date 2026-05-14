use crate::engine::platform;
use crate::engine::scraper::parser::scan_html_for_link;

const USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

/// Scrapes a GitHub repository's release page to find the appropriate download URL
pub fn find_github_asset_url(base_url: &str, tag: Option<&str>) -> Option<String> {
    let terms = platform::get_search_terms();
    let os_term = platform::get_system_info().os.to_lowercase();

    let release_page_url = if let Some(t) = tag {
        format!("{}/releases/tag/{}", base_url, t)
    } else {
        format!("{}/releases/latest", base_url)
    };

    let res = ureq::get(&release_page_url).set("User-Agent", USER_AGENT).call().ok()?;
    let final_url = res.get_url().to_string();
    let mut html = res.into_string().ok()?;
    
    let mut download_link = scan_html_for_link(&html, &terms, &os_term);

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
    
    let full_dl_url = if dl_path.starts_with("http") {
        dl_path
    } else {
        let domain = base_url.split('/').take(3).collect::<Vec<_>>().join("/");
        format!("{}{}{}", domain, if dl_path.starts_with('/') { "" } else { "/" }, dl_path)
    };

    Some(full_dl_url)
}
