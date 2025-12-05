use clap::{Parser, Subcommand};
use color_eyre::eyre::Result;
use std::process::exit;

mod algorand;
mod app_state;
mod boot_screen;
mod commands;
mod tui;
mod ui;
mod updater;
mod widgets;

use app_state::App;
use boot_screen::BootScreen;

const VERSION: &str = env!("CARGO_PKG_VERSION");

const LOGO: &str = r#"
██╗      █████╗ ███████╗██╗   ██╗██╗      ██████╗ ██████╗  █████╗
██║     ██╔══██╗╚══███╔╝╚██╗ ██╔╝██║     ██╔═══██╗██╔══██╗██╔══██╗
██║     ███████║  ███╔╝  ╚████╔╝ ██║     ██║   ██║██████╔╝███████║
██║     ██╔══██║ ███╔╝    ╚██╔╝  ██║     ██║   ██║██╔══██╗██╔══██║
███████╗██║  ██║███████╗   ██║   ███████╗╚██████╔╝██║  ██║██║  ██║
╚══════╝╚═╝  ╚═╝╚══════╝   ╚═╝   ╚══════╝ ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝
"#;

#[derive(Parser)]
#[command(version = VERSION, about, long_about = None, disable_version_flag = true, disable_help_flag = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Update {
        #[arg(short, long)]
        install: bool,
    },

    Version,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if let Some(command) = cli.command {
        match command {
            Commands::Update { install } => match updater::check_for_updates() {
                Ok(Some(latest_version)) => {
                    if install {
                        match updater::update_app() {
                            Ok(_) => {
                                exit(0);
                            }
                            Err(_) => {
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
                    exit(0);
                }
                Err(_) => {
                    exit(1);
                }
            },
            Commands::Version => {
                println!("{}", LOGO);
                println!("LazyLora v{}", VERSION);
                println!("A terminal UI for exploring the Algorand blockchain");
                exit(0);
            }
        }
    }

    // Run boot screen animation before initializing the main app
    // Get terminal size for boot screen
    let (cols, rows) = crossterm::terminal::size().unwrap_or((80, 24));

    // Run boot screen
    let mut boot = BootScreen::new((cols, rows));
    let should_continue = boot.run(|screen, frame| screen.draw(frame)).await;

    match should_continue {
        Ok(false) => {
            // User pressed Ctrl+C during boot
            return Ok(());
        }
        Err(_) => {
            // Boot screen error, continue anyway
        }
        Ok(true) => {
            // Continue to main app
        }
    }

    color_eyre::install()?;

    let mut terminal = tui::init()?;

    let mut app = App::new().await?;
    let app_result = app.run(&mut terminal).await;

    if let Err(_err) = tui::restore() {}

    app_result
}
