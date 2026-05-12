mod subcommands;

use clap::Parser;
use subcommands::Commands;

#[derive(Parser)]
#[command(
    name = "flix",
    version = env!("CARGO_PKG_VERSION"),
    about = "The Blazingly Fast Package Manager",
    disable_version_flag = true,
    arg_required_else_help = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Print version information
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    pub version: Option<bool>,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Install { url, shared, release, default, version } => {
            flix_core::engine::install(&url, shared, release, default, version);
        }

        Commands::List { shared, show_version } => {
            flix_core::engine::list(shared, show_version);
        }

        Commands::Update { name, shared, release } => {
            flix_core::engine::update(name.as_deref(), shared, release);
        }

        Commands::Remove { name, shared } => {
            flix_core::engine::remove(&name, shared);
        }

        Commands::Default { set } => {
            if let Some(path) = set {
                // We will build out a specific config-setter for this later, 
                // but for now let's just acknowledge it.
                println!("⚙️ Feature coming soon: Set default path to {}", path);
            }
        }

        Commands::ShellInit => {
            flix_core::engine::shell_init();
        }

        Commands::Setup => {
            flix_core::engine::self_install();
        }
    }
}
