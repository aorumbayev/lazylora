//! Boot screen module for LazyLora TUI application.
//!
//! This module provides a static boot screen with the LazyLora logo.
//!
//! # Features
//! - Static logo: "LAZY" in green, "LORA" in cyan
//! - Version label display
//! - Horizontally and vertically centered logo
//! - Skip functionality with any key (Ctrl+C exits application)
//! - TTY detection to skip boot screen in non-interactive environments

use color_eyre::Result;
use crossterm::{
    event::{Event, KeyCode, KeyEvent, KeyModifiers, poll, read},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    tty::IsTty,
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, Paragraph},
};
use std::io;
use std::time::{Duration, Instant};
use tokio::time::sleep;

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Boot screen animation phases
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnimationPhase {
    /// Displaying static logo
    Displaying,
    /// Animation complete, ready to proceed
    Complete,
}

/// Boot screen configuration and state for LazyLora TUI.
///
/// Manages the boot sequence with a static logo display.
pub struct BootScreen {
    /// Time when the boot screen started
    start_time: Instant,
    /// Current animation phase
    animation_phase: AnimationPhase,
}

impl BootScreen {
    /// Create a new boot screen with the given terminal dimensions.
    pub fn new(_terminal_size: (u16, u16)) -> Self {
        Self {
            start_time: Instant::now(),
            animation_phase: AnimationPhase::Displaying,
        }
    }

    /// Run boot screen for a fixed duration (no skip).
    /// Only Ctrl+C exits the application.
    ///
    /// # Returns
    /// * `Ok(true)` - Continue to main application
    /// * `Ok(false)` - Exit application (user pressed Ctrl+C)
    /// * `Err(_)` - Terminal error occurred
    pub async fn run_fixed_duration(&mut self, duration: Duration) -> Result<bool> {
        // Skip boot screen if not in interactive TTY
        if !std::io::stdout().is_tty() {
            sleep(duration).await;
            return Ok(true);
        }

        // Setup terminal for boot screen
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.clear()?;

        let frame_duration = Duration::from_millis(100);

        loop {
            let elapsed = self.start_time.elapsed();

            // Handle user input - only Ctrl+C exits
            if poll(Duration::from_millis(0))?
                && let Event::Key(KeyEvent {
                    code, modifiers, ..
                }) = read()?
                && let KeyCode::Char('c') = code
                && modifiers.contains(KeyModifiers::CONTROL)
            {
                disable_raw_mode()?;
                execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
                return Ok(false);
            }

            // Update animation state
            self.update_animation_phase(elapsed);

            // Render current frame
            terminal.draw(|frame| {
                self.draw(frame);
            })?;

            // Check for completion
            if elapsed >= duration {
                break;
            }

            sleep(frame_duration).await;
        }

        // Cleanup terminal
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

        Ok(true)
    }

    /// Run the boot screen animation.
    ///
    /// # Returns
    /// * `Ok(true)` - Continue to main application
    /// * `Ok(false)` - Exit application (user pressed Ctrl+C)
    /// * `Err(_)` - Terminal error occurred
    #[allow(dead_code)]
    pub async fn run<F>(&mut self, draw_fn: F) -> Result<bool>
    where
        F: Fn(&mut Self, &mut Frame) + Send + 'static,
    {
        // Skip boot screen if not in interactive TTY
        if !std::io::stdout().is_tty() {
            return Ok(true);
        }

        // Setup terminal for boot screen
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.clear()?;

        // Boot screen timing
        let boot_duration = Duration::from_millis(1500); // 1.5 seconds
        let frame_duration = Duration::from_millis(100); // Check for input every 100ms

        loop {
            let elapsed = self.start_time.elapsed();

            // Handle user input - skip or exit
            if poll(Duration::from_millis(0))?
                && let Event::Key(KeyEvent {
                    code, modifiers, ..
                }) = read()?
            {
                match code {
                    // Ctrl+C exits the application
                    KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                        disable_raw_mode()?;
                        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
                        return Ok(false);
                    }
                    // Any other key skips the boot screen
                    _ => {
                        break;
                    }
                }
            }

            // Update animation state
            self.update_animation_phase(elapsed);

            // Render current frame
            terminal.draw(|frame| {
                draw_fn(self, frame);
            })?;

            // Check for completion
            if elapsed >= boot_duration || self.animation_phase == AnimationPhase::Complete {
                break;
            }

            sleep(frame_duration).await;
        }

        // Cleanup terminal
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

        Ok(true)
    }

    /// Update the animation phase based on elapsed time.
    fn update_animation_phase(&mut self, elapsed: Duration) {
        self.animation_phase = if elapsed.as_millis() < 1500 {
            AnimationPhase::Displaying
        } else {
            AnimationPhase::Complete
        };
    }

    /// Draw the boot screen based on current animation phase.
    pub fn draw(&self, frame: &mut Frame) {
        let area = frame.area();

        // Clear background
        frame.render_widget(Clear, area);

        self.draw_static_logo(frame, area);
    }

    /// Draw the static logo.
    /// "LAZY" in green, "LORA" in cyan.
    fn draw_static_logo(&self, frame: &mut Frame, area: Rect) {
        let ascii_lines = self.get_static_ascii_art();

        // Calculate logo dimensions
        let logo_height = ascii_lines.len() as u16 + 2; // +2 for version line and spacing
        let logo_width = 67u16; // Width of the full ASCII art

        // Center vertically
        let vertical_padding = area.height.saturating_sub(logo_height) / 2;

        // Center horizontally
        let horizontal_padding = area.width.saturating_sub(logo_width) / 2;

        // Create centered area
        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(vertical_padding),
                Constraint::Length(logo_height),
                Constraint::Min(0),
            ])
            .split(area);

        let horizontal_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(horizontal_padding),
                Constraint::Length(logo_width),
                Constraint::Min(0),
            ])
            .split(vertical_chunks[1]);

        let logo_area = horizontal_chunks[1];

        // Split logo area for ASCII art and version
        let logo_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(ascii_lines.len() as u16),
                Constraint::Length(1), // spacing
                Constraint::Length(1), // version
            ])
            .split(logo_area);

        // Render ASCII art
        let ascii_paragraph = Paragraph::new(ascii_lines).alignment(Alignment::Center);
        frame.render_widget(ascii_paragraph, logo_chunks[0]);

        // Render version
        let version_line = Line::from(vec![Span::styled(
            format!("v{}", VERSION),
            Style::default()
                .fg(Color::Indexed(8)) // Dim gray
                .add_modifier(Modifier::ITALIC),
        )]);
        let version_paragraph = Paragraph::new(version_line).alignment(Alignment::Center);
        frame.render_widget(version_paragraph, logo_chunks[2]);
    }

    /// Get static ASCII art with fixed colors.
    /// LAZY in green, LORA in cyan.
    fn get_static_ascii_art(&self) -> Vec<Line<'static>> {
        let lazy_style = Style::default()
            .fg(Color::Indexed(10)) // Bright green
            .add_modifier(Modifier::BOLD);

        let lora_style = Style::default()
            .fg(Color::Indexed(14)) // Cyan
            .add_modifier(Modifier::BOLD);

        // Original logo split at position 34 (after LAZY, before LORA's L)
        vec![
            Line::from(vec![
                Span::styled("██╗      █████╗ ███████╗██╗   ██╗", lazy_style),
                Span::styled("██╗      ██████╗ ██████╗  █████╗ ", lora_style),
            ]),
            Line::from(vec![
                Span::styled("██║     ██╔══██╗╚══███╔╝╚██╗ ██╔╝", lazy_style),
                Span::styled("██║     ██╔═══██╗██╔══██╗██╔══██╗", lora_style),
            ]),
            Line::from(vec![
                Span::styled("██║     ███████║  ███╔╝  ╚████╔╝ ", lazy_style),
                Span::styled("██║     ██║   ██║██████╔╝███████║", lora_style),
            ]),
            Line::from(vec![
                Span::styled("██║     ██╔══██║ ███╔╝    ╚██╔╝  ", lazy_style),
                Span::styled("██║     ██║   ██║██╔══██╗██╔══██║", lora_style),
            ]),
            Line::from(vec![
                Span::styled("███████╗██║  ██║███████╗   ██║   ", lazy_style),
                Span::styled("███████╗╚██████╔╝██║  ██║██║  ██║", lora_style),
            ]),
            Line::from(vec![
                Span::styled("╚══════╝╚═╝  ╚═╝╚══════╝   ╚═╝   ", lazy_style),
                Span::styled("╚══════╝ ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝", lora_style),
            ]),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boot_screen_creation() {
        let screen = BootScreen::new((80, 24));
        assert_eq!(screen.animation_phase, AnimationPhase::Displaying);
    }

    #[test]
    fn test_animation_phase_transitions() {
        let mut screen = BootScreen::new((80, 24));

        // Test Displaying phase
        screen.update_animation_phase(Duration::from_millis(1000));
        assert_eq!(screen.animation_phase, AnimationPhase::Displaying);

        // Test Complete phase
        screen.update_animation_phase(Duration::from_millis(1600));
        assert_eq!(screen.animation_phase, AnimationPhase::Complete);
    }
}
