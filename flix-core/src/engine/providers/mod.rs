pub mod generic;
pub mod github;
pub mod traits;

pub use traits::Provider;

/// A simple router that looks at a URL and returns the appropriate Provider struct.
pub fn get_provider(url: &str) -> Box<dyn Provider> {
    let github = github::GithubProvider;
    
    if github.supports(url) {
        Box::new(github)
    } else {
        Box::new(generic::GenericProvider)
    }
}
