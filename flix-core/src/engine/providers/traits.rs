pub trait Provider {
    /// Returns the name of the provider (e.g., "GitHub", "Generic Git")
    fn name(&self) -> &'static str;

    /// Checks if this provider is built to handle the given URL domain.
    fn supports(&self, url: &str) -> bool;

    /// Reaches out to the specific site's API to find a pre-built binary.
    /// Returns None if unsupported or not found.
    fn find_asset_url(&self, repo_url: &str, git_ref: Option<&str>) -> Option<String>;
    
    /// Extracts the version tag (like "v1.5.1") from the site's asset URL or API.
    fn extract_tag(&self, asset_url: &str) -> Option<String>;
}
