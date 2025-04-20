use clap::{Parser, Subcommand};
use color_eyre::eyre::{Result, eyre};
use std::process::exit;

mod algorand;
mod app_state;
mod tui;
mod ui;
mod updater;

use app_state::App;

// LazyLora version from Cargo.toml
const VERSION: &str = env!("CARGO_PKG_VERSION");

// ASCII art logo
const LOGO: &str = r#"
██╗      █████╗ ███████╗██╗   ██╗██╗      ██████╗ ██████╗  █████╗
██║     ██╔══██╗╚══███╔╝╚██╗ ██╔╝██║     ██╔═══██╗██╔══██╗██╔══██╗
██║     ███████║  ███╔╝  ╚████╔╝ ██║     ██║   ██║██████╔╝███████║
██║     ██╔══██║ ███╔╝    ╚██╔╝  ██║     ██║   ██║██╔══██╗██╔══██║
███████╗██║  ██║███████╗   ██║   ███████╗╚██████╔╝██║  ██║██║  ██║
╚══════╝╚═╝  ╚═╝╚══════╝   ╚═╝   ╚══════╝ ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝
"#;

/// LazyLora - Terminal UI for Algorand blockchain
#[derive(Parser)]
#[command(version = VERSION, about, long_about = None)]
struct Cli {
    /// Subcommand to run
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Check for updates
    Update {
        /// Install the latest version if available
        #[arg(short, long)]
        install: bool,
    },
    /// Display version with ASCII art
    Version,
}

/// Application entry point
fn main() -> Result<()> {
    let cli = Cli::parse();

    // Handle CLI commands
    if let Some(command) = cli.command {
        match command {
            Commands::Update { install } => {
                let update_available = updater::check_for_updates(VERSION).map_err(|e| eyre!(e))?;

                if update_available && install {
                    updater::update_app().map_err(|e| eyre!(e))?;
                    println!("Update completed successfully!");
                    exit(0);
                } else if update_available {
                    println!("Run with --install flag to install the update.");
                    exit(0);
                } else {
                    println!("No updates available.");
                    exit(0);
                }
            }
            Commands::Version => {
                println!("{}", LOGO);
                println!("LazyLora v{}", VERSION);
                println!("A terminal UI for exploring the Algorand blockchain");
                exit(0);
            }
        }
    }

    // Run normal application flow
    color_eyre::install()?;
    let mut terminal = tui::init()?;

    // Create and run app
    let app_result = App::new().run(&mut terminal);

    // Restore terminal
    if let Err(err) = tui::restore() {
        eprintln!("Failed to restore terminal: {}", err);
    }

    // Return the app result
    app_result
}
