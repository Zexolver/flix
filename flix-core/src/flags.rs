use clap::Args;
use serde::{Deserialize, Serialize};

#[derive(Args, Debug, Clone, Serialize, Deserialize)]
pub struct SharedArgs {
    /// Suppress standard build output (e.g., cargo noise)
    #[arg(short = 'q', long)]
    pub quiet: bool,

    /// Overwrite existing binaries or force a fresh build
    #[arg(short = 'f', long)]
    pub force: bool,

    /// Skip interactive prompts and use defaults
    #[arg(short = 'y', long)]
    pub yes: bool,

    /// Comma-separated list of tags for filtering or categorizing
    #[arg(short = 't', long, value_delimiter = ',')]
    pub tags: Vec<String>,

    /// Override the installation or search path for this command
    #[arg(short = 'p', long)]
    pub path: Option<String>,
}
