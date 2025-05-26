use std::io::{self, Stdout};

use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    crossterm::{
        event::{DisableMouseCapture, EnableMouseCapture},
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
};

pub type Tui = Terminal<CrosstermBackend<Stdout>>;

pub fn init() -> io::Result<Tui> {
    enable_raw_mode()?;
    execute!(std::io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;

    set_panic_hook();

    let backend = CrosstermBackend::new(std::io::stdout());
    Terminal::new(backend)
}

fn set_panic_hook() {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = restore();
        hook(panic_info);
    }));
}

pub fn restore() -> io::Result<()> {
    execute!(std::io::stdout(), DisableMouseCapture, LeaveAlternateScreen)?;
    disable_raw_mode()
}
