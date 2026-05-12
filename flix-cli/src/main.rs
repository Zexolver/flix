use clap::{Parser, Subcommand};
use flix_core::engine;

#[derive(Parser)]
#[command(
    name = "flix", 
    version = "0.1.0", 
    about = "Friendly, Lightweight, Xtensible installer", 
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Install a repository from a Git URL
    Install {
        /// URL of the git repository
        url: String,
        
        /// Automatically use the default branch (skip prompts)
        #[arg(short = 'd', long)]
        default: bool,
        
        /// Suppress output from build commands
        #[arg(short = 'q', long)]
        quiet: bool,
        
        /// Auto-confirm any installation prompts
        #[arg(short = 'y', long)]
        yes: bool,
        
        /// Add custom tags (can be used multiple times, e.g., -t rice -t dev)
        #[arg(short = 't', long)]
        tag: Vec<String>,
        
        /// Override the default installation path
        #[arg(short = 'p', long)]
        path: Option<String>,
    },
    
    /// Remove an installed package
    Remove {
        /// Name of the package to remove
        name: String,
    },
    
    /// Update installed packages
    Update {
        /// Specific package to update (updates all if omitted)
        name: Option<String>,
    },
    
    /// List installed packages
    List {
        /// Filter the list by a specific tag
        #[arg(short = 't', long)]
        tag: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    // Route the CLI commands to the core engine
    match &cli.command {
        Commands::Install { url, default, quiet, yes, tag, path } => {
            engine::install(url, *default, *quiet, *yes, tag, path.as_deref());
        }
        Commands::Remove { name } => {
            engine::remove(name);
        }
        Commands::Update { name } => {
            engine::update(name.as_deref());
        }
        Commands::List { tag } => {
            engine::list(tag.as_deref());
        }
    }
}
