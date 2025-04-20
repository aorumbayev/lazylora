use clap::{Parser, Subcommand};
use color_eyre::eyre::Result;
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
#[command(version = VERSION, about, long_about = None, disable_version_flag = true, disable_help_flag = true)]
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

    if let Some(command) = cli.command {
        match command {
            Commands::Update { install } => {
                match updater::check_for_updates() {
                    Ok(Some(latest_version)) => {
                        // Update available
                        if install {
                            match updater::update_app() {
                                Ok(_) => {
                                    // Success message handled in update_app
                                    exit(0);
                                }
                                Err(e) => {
                                    eprintln!("Update failed: {}", e);
                                    exit(1);
                                }
                            }
                        } else {
                            println!(
                                "Update available: {}. Run with --install flag to install.",
                                latest_version
                            );
                            exit(0);
                        }
                    }
                    Ok(None) => {
                        // No update available
                        // Message handled in check_for_updates
                        exit(0);
                    }
                    Err(e) => {
                        eprintln!("Failed to check for updates: {}", e);
                        exit(1);
                    }
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
