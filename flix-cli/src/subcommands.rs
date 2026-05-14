use clap::Subcommand;
use flix_core::flags::SharedArgs;

#[derive(Subcommand)]
pub enum Commands {
    /// Install a package from a git URL or search for releases
    Install {
        url: String,

        #[command(flatten)]
        shared: SharedArgs,

        /// Search for pre-built binaries on GitHub Releases first
        #[arg(short = 'r', long)]
        release: bool,

        /// Mark this package as the primary/default binary
        #[arg(short = 'd', long)]
        default: bool,

        /// Install a specific git tag or commit hash (e.g., -V v0.1.0)
        #[arg(short = 'V', long = "git-ref")]
        version: Option<String>,
    },

    /// Remove an installed package
    Remove {
        name: String,
        
        #[command(flatten)]
        shared: SharedArgs,
    },

    /// List all installed packages
    List {
        #[command(flatten)]
        shared: SharedArgs,

        /// Show installed versions/hashes in the output
        #[arg(short = 'v', long)]
        show_version: bool,
    },

    /// Update installed packages to their latest versions
    Update {
        /// Optional: Name of a specific package to update
        name: Option<String>,

        #[command(flatten)]
        shared: SharedArgs,

        /// Prefer pre-built binary updates if available
        #[arg(short = 'r', long)]
        release: bool,
    },

    /// Configure the global installation directory
    Default {
        /// The new path to set as the global default
        #[arg(short = 's', long)]
        set: Option<String>,
    },

    /// Configure shell PATH (bash/zsh/profile)
    ShellInit,

    /// Run the interactive first-time setup
    Setup,
    
    /// Internal: Generate static completion scripts
    #[command(hide = true)]
    GenerateCompletion {
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },

    /// Internal: List installed package names for dynamic completion
    #[command(name = "_list-installed", hide = true)]
    _ListInstalled,
}
