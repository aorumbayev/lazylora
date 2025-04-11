

use color_eyre::Result;

mod algorand;
mod app_state;
mod tui;
mod ui;

use app_state::App;

/// Application entry point
fn main() -> Result<()> {
    // Setup terminal
    color_eyre::install()?;
    let mut terminal = tui::init()?;

    // Create and run app
    let app_result = App::new().run(&mut terminal);

    // Restore terminal
    if let Err(err) = tui::restore() {
        eprintln!(
            "Failed to restore terminal. Run `reset` or restart your terminal to recover: {}",
            err
        );
    }

    // Return the app result
    app_result
}
