mod subcommands;

use clap::Parser;
use subcommands::Commands;

#[derive(Parser)]
#[command(
    name = "flix",
    // This fixes the panic by giving the Version action something to print
    version = env!("CARGO_PKG_VERSION"),
    about = "The Blazingly Fast Package Manager",
    disable_version_flag = true, // Allows us to use -v for version and -V for git-hash
    arg_required_else_help = true
)]
struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Print version information
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    pub version: bool,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        // Prefixed unused variables with underscores to silence compiler warnings
        Commands::Install { 
            url, 
            shared, 
            release, 
            default: _, 
            version: _ 
        } => {
            println!("🚀 Installing {}...", url);
            println!("   Force:   {}", shared.force);
            println!("   Quiet:   {}", shared.quiet);
            println!("   Binary:  {}", release);
            println!("   Tags:    {:?}", shared.tags);
            
            // Logic for engine::install will go here next
        }

        Commands::List { shared, show_version } => {
            println!("📋 Listing packages...");
            println!("   Filter Path: {:?}", shared.path);
            println!("   Filter Tags: {:?}", shared.tags);
            println!("   Show Hashes: {}", show_version);

            // Logic for engine::list will go here next
        }

        Commands::Update { name, shared, release } => {
            match name {
                Some(n) => println!("🔄 Updating '{}' (Force: {})", n, shared.force),
                None => println!("🔄 Updating all packages (Binary Mode: {})", release),
            }
        }

        Commands::Remove { name, shared } => {
            println!("🗑️ Removing {} (Auto-confirm: {})", name, shared.yes);
        }

        Commands::Default { set } => {
            if let Some(path) = set {
                println!("⚙️ Setting global install path to: {}", path);
            }
        }

        Commands::ShellInit => {
            println!("🐚 Initializing shell configuration...");
        }

        Commands::Setup => {
            println!("🛠️ Running first-time setup...");
        }
    }
}
