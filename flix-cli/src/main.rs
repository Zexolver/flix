use clap::{Parser, Subcommand};
use flix_core::engine;

#[derive(Parser)]
#[command(name = "flix")]
#[command(about = "An organized, blazingly fast package manager", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Install a package from a git URL
    Install {
        url: String,

        /// Search for pre-built release binaries first
        #[arg(short, long)]
        release: bool,

        /// Mark as default package
        #[arg(short, long)]
        default: bool,

        /// Reduce output verbosity
        #[arg(short, long)]
        quiet: bool,

        /// Skip all prompts (automatic 'yes')
        #[arg(short, long)]
        yes: bool,

        /// Assign tags to this package (comma-separated)
        #[arg(short, long, value_delimiter = ',')]
        tags: Vec<String>,

        /// Specify a custom installation path for this package
        #[arg(short, long)]
        path: Option<String>,
    },

    /// Remove an installed package and its binary
    Remove { 
        name: String 
    },

    /// Update an existing package or all packages
    Update { 
        name: Option<String> 
    },

    /// List all packages managed by flix
    List {
        /// Filter packages by tag
        #[arg(short, long)]
        tag: Option<String>,
    },

    /// Manage the global default installation directory
    Default {
        /// Set a new global default path
        #[arg(short, long)]
        set: Option<String>,
    },

    /// Automate adding the flix bin path to your shell profile
    ShellInit,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Install { 
            url, 
            release, 
            default, 
            quiet, 
            yes, 
            tags, 
            path 
        } => {
            engine::install(
                &url, 
                release, 
                default, 
                quiet, 
                yes, 
                &tags, 
                path.as_deref()
            );
        }

        Commands::Remove { name } => {
            engine::remove(&name);
        }

        Commands::Update { name } => {
            engine::update(name.as_deref());
        }

        Commands::List { tag } => {
            engine::list(tag.as_deref());
        }

        Commands::Default { set } => {
            if let Some(new_path) = set {
                engine::set_default_path(&new_path);
            } else {
                let config = flix_core::config::load_config();
                match config.default_install_path {
                    Some(p) => println!("📍 Current global default: {}", p.display()),
                    None => println!("⚠️ No default path set. Run an install or use 'flix default --set <PATH>'."),
                }
            }
        }

        Commands::ShellInit => {
            engine::shell_init();
        }
    }
}
