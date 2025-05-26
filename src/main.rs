use clap::{Parser, Subcommand};
use color_eyre::eyre::Result;
use std::process::exit;

mod algorand;
mod app_state;
mod tui;
mod ui;
mod updater;

use app_state::App;

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

    color_eyre::install()?;

    let mut terminal = tui::init()?;

    let mut app = App::new().await?;
    let app_result = app.run(&mut terminal).await;

    if let Err(_err) = tui::restore() {}

    app_result
}
