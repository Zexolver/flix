mod subcommands;

use clap::{Parser, CommandFactory};
use clap_complete::generate;
use std::io;
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

        Commands::List { shared } => {
            flix_core::engine::list(shared);
        }
        
        Commands::Tag { name, add, remove } => {
            match flix_core::engine::manage_tags(&name, add, remove) {
                Ok((added_tags, removed_tags)) => {
                    for t in &removed_tags {
                        println!("➖ Removed tag '{}' from '{}'", t, name);
                    }
                    for t in &added_tags {
                        println!("➕ Added tag '{}' to '{}'", t, name);
                    }
                    
                    if added_tags.is_empty() && removed_tags.is_empty() {
                        println!("ℹ️ No changes made to tags for '{}'.", name);
                    } else {
                        println!("✅ Tags updated for '{}'.", name);
                    }
                }
                Err(e) => {
                    eprintln!("❌ Error: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Update { name, shared, release } => {
            flix_core::engine::update(name.as_deref(), shared, release);
        }

        Commands::Remove { name, shared } => {
            flix_core::engine::remove(&name, shared);
        }

        Commands::Default { set } => {
            if let Some(path) = set {
                println!("⚙️ Feature coming soon: Set default path to {}", path);
            }
        }

        Commands::ShellInit => {
            flix_core::engine::shell_init();
        }

        Commands::Setup => {
            flix_core::engine::self_install();
        }

        Commands::GenerateCompletion { shell } => {
            let mut cmd = Cli::command();
            let name = cmd.get_name().to_string();
            generate(shell, &mut cmd, name, &mut io::stdout());
        }

        Commands::_ListInstalled => {
            let config = flix_core::config::load_config();
            for name in config.packages.keys() {
                println!("{}", name);
            }
        }
    }
}
