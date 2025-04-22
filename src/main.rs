use clap::{Parser, Subcommand};
use color_eyre::eyre::Result;
use env_logger;
use std::time::{Duration, Instant};
use tokio::sync::mpsc; // Use tokio mpsc for async communication

// Declare modules
mod algorand;
mod app; // Renamed from app_state
mod components;
mod config;
mod constants;
mod event;
mod handler;
mod network;
mod tui;
mod ui;
mod updater;

// Use new types
use crate::{
    algorand::AlgoClient,
    app::App,
    constants::TICK_RATE,
    event::{Action, NetworkUpdateEvent},
    handler::handle_event,
    network::NetworkManager,
    tui::Tui,
};

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

    // Add a general help flag
    #[arg(short, long, action = clap::ArgAction::HelpLong)]
    help: Option<bool>,
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
#[tokio::main] // Use tokio main for async runtime
async fn main() -> Result<()> {
    // Initialize logger FIRST
    env_logger::init();

    // Command line argument parsing
    let cli = Cli::parse();
    if handle_cli_commands(&cli).await? {
        return Ok(()); // Exit if a command was handled
    }

    // Setup application
    color_eyre::install()?;
    let mut terminal = tui::init()?;
    let mut app = App::new();

    // Setup Tokio runtime and channels for async communication
    let runtime = tokio::runtime::Handle::current(); // Get handle to current runtime

    // Channel for network events -> main loop
    let (network_event_sender, mut network_event_receiver) =
        mpsc::channel::<NetworkUpdateEvent>(100);

    // Create shared AlgoClient for NetworkManager
    let initial_client = AlgoClient::new(app.settings.selected_network.clone());
    let shared_client = Arc::new(tokio::sync::Mutex::new(initial_client));

    // Create NetworkManager
    let network_manager = NetworkManager::new(
        Arc::clone(&shared_client),
        Arc::clone(&app.show_live),
        Arc::clone(&app.blocks),
        Arc::clone(&app.transactions),
        runtime.clone(),
        network_event_sender,
    );

    // Start background network tasks
    network_manager.start_background_loop();

    // Fetch initial data
    network_manager.fetch_initial_data(app.settings.selected_network.as_str().to_string());

    // Run the main application loop
    run_app(
        &mut terminal,
        &mut app,
        &network_manager,
        &mut network_event_receiver,
    )
    .await?;

    // Restore terminal
    tui::restore()?;
    Ok(())
}

/// Handles CLI subcommands like --version or --update.
/// Returns Ok(true) if a command was handled and the app should exit, Ok(false) otherwise.
async fn handle_cli_commands(cli: &Cli) -> Result<bool> {
    if cli.help.is_some() {
        // Let clap print help
        return Ok(true);
    }

    if let Some(command) = &cli.command {
        match command {
            Commands::Update { install } => {
                match updater::check_for_updates() {
                    Ok(Some(latest_version)) => {
                        println!("Update available: {}", latest_version);
                        if *install {
                            println!("Attempting to install...");
                            match updater::update_app() {
                                Ok(_) => println!("Update successful!"),
                                Err(e) => eprintln!("Update failed: {}", e),
                            }
                        } else {
                            println!("Run with '--install' flag to install.");
                        }
                    }
                    Ok(None) => println!("Already using the latest version."),
                    Err(e) => eprintln!("Failed to check for updates: {}", e),
                }
                return Ok(true); // Exit after handling update command
            }
            Commands::Version => {
                println!("{}", LOGO);
                println!("LazyLora v{}", VERSION);
                println!("A terminal UI for exploring the Algorand blockchain");
                return Ok(true); // Exit after showing version
            }
        }
    }
    Ok(false) // No command handled, continue to main app
}

/// Main application loop.
async fn run_app(
    terminal: &mut Tui,
    app: &mut App,
    network_manager: &NetworkManager,
    network_event_receiver: &mut mpsc::Receiver<NetworkUpdateEvent>,
) -> Result<()> {
    let mut last_tick = Instant::now();

    loop {
        if app.exit {
            break;
        }

        // --- Drawing ---
        terminal.draw(|frame| ui::render(app, frame))?;

        // --- Event Handling ---
        // Calculate duration until next tick, ensuring it's not negative.
        let _time_until_next_tick = TICK_RATE
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        // Poll for terminal events first with a very small timeout (e.g., 1ms)
        // Poll for terminal events first with a very small timeout,
        // then check network events and sleep if necessary.
        let mut terminal_event_ready = false;
        if crossterm::event::poll(Duration::from_millis(1))? {
            terminal_event_ready = true;
        }

        // Process terminal event if ready
        if terminal_event_ready {
            match crossterm::event::read() {
                Ok(event) => {
                    // Handle terminal resize immediately
                    if let crossterm::event::Event::Resize(width, height) = event {
                        app.update_terminal_size(width, height);
                        // Redraw happens implicitly at start of next loop iteration
                        continue; // Skip rest of loop iteration
                    }
                    // Handle other terminal events (keys, mouse)
                    if let Some(action) = handle_event(app, event) {
                        if let Err(e) = app.update(action, network_manager) {
                            app.update(
                                Action::ShowMessage(format!("Error: {}", e)),
                                network_manager,
                            )?;
                        }
                    }
                }
                Err(_) => {
                    // crossterm read error
                    app.exit = true;
                }
            }
        }

        // Check for network events non-blockingly
        match network_event_receiver.try_recv() {
            Ok(network_event) => {
                // Convert network event into an Action and dispatch it
                let action = match network_event {
                    NetworkUpdateEvent::StatusUpdate(res) => Action::UpdateNetworkStatus(res),
                    NetworkUpdateEvent::BlocksFetched(res) => Action::UpdateBlocks(res),
                    NetworkUpdateEvent::TransactionsFetched(res) => Action::UpdateTransactions(res),
                    NetworkUpdateEvent::SearchResults(res) => Action::UpdateSearchResults(res),
                };
                if let Err(e) = app.update(action, network_manager) {
                    app.update(
                        Action::ShowMessage(format!("Error: {}", e)),
                        network_manager,
                    )?;
                }
            }
            Err(mpsc::error::TryRecvError::Empty) => {
                // No network event currently waiting
            }
            Err(mpsc::error::TryRecvError::Disconnected) => {
                // Channel closed, maybe exit?
                app.exit = true;
            }
        }

        // Update last tick time if tick rate has passed
        if last_tick.elapsed() >= TICK_RATE {
            last_tick = Instant::now();
            // Potentially dispatch Action::Tick if needed
            // if let Err(e) = app.update(Action::Tick, network_manager)? { ... }
        }

        // Small sleep to prevent high CPU usage if no events are pending
        // Only sleep if no terminal event was processed in this iteration
        if !terminal_event_ready {
            let remaining_timeout = TICK_RATE
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_millis(5)); // Sleep for at least 5ms
            tokio::time::sleep(remaining_timeout.min(Duration::from_millis(50))).await; // Cap sleep time
        }
    }
    Ok(())
}

// Needed for Arc::clone inside async block
use std::sync::Arc;
