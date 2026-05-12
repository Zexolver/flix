use clap::{Parser, Subcommand};
mod subcommands;

#[derive(Parser)]
#[command(name = "flix", about = "The Blazingly Fast Package Manager", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Install a package from a git URL
    Install {
        url: String,
        #[arg(short, long)]
        force: bool,
        #[arg(short, long)]
        tags: Vec<String>,
    },
    /// Remove an installed package
    Remove {
        name: String,
        #[arg(short, long)]
        yes: bool,
    },
    /// List all installed packages
    List {
        #[arg(short, long)]
        tag: Option<String>,
    },
    /// Update packages
    Update {
        name: Option<String>,
    },
    /// Configure shell PATH
    ShellInit,
    /// Install Flix to the system
    Setup,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Install { url, force, tags } => subcommands::handle_install(&url, force, tags),
        Commands::Remove { name, yes } => subcommands::handle_remove(&name, yes),
        Commands::List { tag } => flix_core::engine::list(tag.as_deref()),
        Commands::Update { name } => flix_core::engine::update(name.as_deref(), false),
        Commands::ShellInit => subcommands::handle_shell_init(),
        Commands::Setup => subcommands::handle_setup(),
    }
}
