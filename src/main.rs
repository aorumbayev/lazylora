use clap::{Parser, Subcommand, ValueEnum};
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

use algorand::Network;
use app_state::{App, StartupOptions, StartupSearch};
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

/// Network selection for CLI
#[derive(Debug, Clone, Copy, ValueEnum)]
enum NetworkArg {
    Mainnet,
    Testnet,
    Localnet,
}

impl From<NetworkArg> for Network {
    fn from(arg: NetworkArg) -> Self {
        match arg {
            NetworkArg::Mainnet => Network::MainNet,
            NetworkArg::Testnet => Network::TestNet,
            NetworkArg::Localnet => Network::LocalNet,
        }
    }
}

#[derive(Parser)]
#[command(version = VERSION, about, long_about = None, disable_version_flag = true, disable_help_flag = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Transaction ID to look up
    #[arg(short = 't', long = "tx")]
    transaction: Option<String>,

    /// Account address to look up
    #[arg(short = 'a', long = "account")]
    account: Option<String>,

    /// Block number to look up
    #[arg(short = 'b', long = "block")]
    block: Option<u64>,

    /// Asset ID to look up
    #[arg(short = 's', long = "asset")]
    asset: Option<u64>,

    /// Network to connect to
    #[arg(short = 'n', long = "network", value_enum)]
    network: Option<NetworkArg>,

    /// Open transaction in graph view
    #[arg(short = 'g', long = "graph")]
    graph: bool,
}

impl Cli {
    fn into_startup_options(self) -> StartupOptions {
        let search = if let Some(tx) = self.transaction {
            Some(StartupSearch::Transaction(tx))
        } else if let Some(account) = self.account {
            Some(StartupSearch::Account(account))
        } else if let Some(block) = self.block {
            Some(StartupSearch::Block(block))
        } else {
            self.asset.map(StartupSearch::Asset)
        };

        StartupOptions {
            network: self.network.map(Network::from),
            search,
            graph_view: self.graph,
        }
    }
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
            Commands::Update { install } => {
                let source = updater::detect_install_source();
                println!("Installation source: {}", source);

                // Run blocking self_update operations in a dedicated thread
                // to avoid conflicts with the tokio async runtime
                let check_result = tokio::task::spawn_blocking(updater::check_for_updates)
                    .await
                    .expect("Failed to spawn blocking task");

                match check_result {
                    Ok(Some(latest_version)) => {
                        println!("Update available: {}", latest_version);

                        if install {
                            // Check if this installation source supports self-update
                            if source.supports_self_update() {
                                let update_result =
                                    tokio::task::spawn_blocking(updater::update_app)
                                        .await
                                        .expect("Failed to spawn blocking task");

                                match update_result {
                                    Ok(()) => exit(0),
                                    Err(e) => {
                                        eprintln!("Update failed: {}", e);
                                        exit(1)
                                    }
                                }
                            } else {
                                // Package manager installation - provide instructions
                                println!("\nThis installation does not support automatic updates.");
                                if let Some(instructions) = source.update_instructions() {
                                    println!("To update, run:\n  {}", instructions);
                                }
                                exit(0);
                            }
                        } else {
                            // Just checking for updates, provide appropriate guidance
                            if source.supports_self_update() {
                                println!("Run with --install flag to install the update.");
                            } else if let Some(instructions) = source.update_instructions() {
                                println!("To update, run:\n  {}", instructions);
                            }
                            exit(0);
                        }
                    }
                    Ok(None) => exit(0),
                    Err(e) => {
                        eprintln!("Failed to check for updates: {}", e);
                        exit(1)
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

    let startup_options = cli.into_startup_options();

    let (cols, rows) = crossterm::terminal::size().unwrap_or((80, 24));
    let mut boot = BootScreen::new((cols, rows));
    let should_continue = boot.run(|screen, frame| screen.draw(frame)).await;

    if let Ok(false) = should_continue {
        return Ok(());
    }

    color_eyre::install()?;

    let mut terminal = tui::init()?;
    let mut app = App::new(startup_options).await?;
    let app_result = app.run(&mut terminal).await;

    let _ = tui::restore();
    app_result
}
