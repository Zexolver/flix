use crate::engine::providers::traits::Provider;

/// A catch-all provider for custom or unknown Git hosts.
pub struct GenericProvider;

impl Provider for GenericProvider {
    fn name(&self) -> &'static str {
        "Generic Git"
    }

    fn supports(&self, _url: &str) -> bool {
        true 
    }

    fn find_asset_url(&self, _repo_url: &str, _git_ref: Option<&str>) -> Option<String> {
        None 
    }

    fn extract_tag(&self, _asset_url: &str) -> Option<String> {
        None
    }

    fn get_latest_tag(&self, _repo_url: &str) -> Option<String> {
        None
    }
}
