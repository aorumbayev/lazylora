//! Boot screen module for LazyLora TUI application.
//!
//! This module provides an animated boot screen with a pulse animation effect.
//!
//! # Features
//! - Pulse animation: "LAZY" stays green, "LORA" pulses from white to cyan
//! - Version label display
//! - Horizontally and vertically centered logo
//! - 60 FPS smooth animation
//! - Skip functionality with any key (Ctrl+C exits application)
//! - TTY detection to skip boot screen in non-interactive environments

use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, poll, read};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, Paragraph},
};
use std::time::{Duration, Instant};
use tokio::time::sleep;

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Boot screen animation phases
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnimationPhase {
    /// Main pulsing animation
    Pulsing,
    /// Animation complete, ready to proceed
    Complete,
}

/// Boot screen configuration and state for LazyLora TUI.
///
/// Manages the animated boot sequence with pulse effect.
pub struct BootScreen {
    /// Time when the boot screen started
    start_time: Instant,
    /// Current animation phase
    animation_phase: AnimationPhase,
    /// Frame counter for smooth animations
    animation_frame: u32,
    /// Terminal dimensions for responsive scaling
    #[allow(dead_code)]
    terminal_size: (u16, u16),
}

impl BootScreen {
    /// Create a new boot screen with the given terminal dimensions.
    pub fn new(terminal_size: (u16, u16)) -> Self {
        Self {
            start_time: Instant::now(),
            animation_phase: AnimationPhase::Pulsing,
            animation_frame: 0,
            terminal_size,
        }
    }

    /// Run the boot screen animation.
    ///
    /// # Returns
    /// * `Ok(true)` - Continue to main application
    /// * `Ok(false)` - Exit application (user pressed Ctrl+C)
    /// * `Err(_)` - Terminal error occurred
    pub async fn run<F>(&mut self, draw_fn: F) -> Result<bool>
    where
        F: Fn(&mut Self, &mut Frame) + Send + 'static,
    {
        use crossterm::{
            execute,
            terminal::{
                EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
            },
            tty::IsTty,
        };
        use ratatui::{Terminal, backend::CrosstermBackend};
        use std::io;

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

        // Animation timing configuration
        let boot_duration = Duration::from_millis(2000); // 2 seconds total
        let frame_duration = Duration::from_millis(16); // ~60 FPS

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
            self.animation_frame = self.animation_frame.wrapping_add(1);

            // Render current frame
            terminal.draw(|frame| {
                draw_fn(self, frame);
            })?;

            // Check for animation completion
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
        self.animation_phase = if elapsed.as_millis() < 2000 {
            AnimationPhase::Pulsing
        } else {
            AnimationPhase::Complete
        };
    }

    /// Draw the boot screen based on current animation phase.
    pub fn draw(&self, frame: &mut Frame) {
        let area = frame.area();

        // Clear background
        frame.render_widget(Clear, area);

        match self.animation_phase {
            AnimationPhase::Pulsing => self.draw_pulsing(frame, area),
            AnimationPhase::Complete => self.draw_pulsing(frame, area),
        }
    }

    /// Draw the pulsing logo animation.
    /// "LAZY" stays constant green, "LORA" pulses from white to cyan.
    fn draw_pulsing(&self, frame: &mut Frame, area: Rect) {
        // Calculate pulse value using sine wave (0.0 to 1.0)
        let time = self.animation_frame as f32 * 0.1;
        let pulse = (time.sin() + 1.0) / 2.0;

        // Get ASCII art lines with pulse effect
        let ascii_lines = self.get_pulsing_ascii_art(pulse);

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

    /// Get ASCII art with pulse effect on LORA part.
    /// LAZY stays constant green, LORA pulses from white to cyan.
    fn get_pulsing_ascii_art(&self, pulse: f32) -> Vec<Line<'static>> {
        // Interpolate LORA color from white (15) to cyan (14) based on pulse
        let lora_color = if pulse > 0.5 {
            Color::Indexed(14) // Cyan
        } else {
            Color::Indexed(15) // White
        };

        let lazy_style = Style::default()
            .fg(Color::Indexed(10)) // Bright green
            .add_modifier(Modifier::BOLD);

        let lora_style = Style::default().fg(lora_color).add_modifier(Modifier::BOLD);

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
        assert_eq!(screen.animation_phase, AnimationPhase::Pulsing);
        assert_eq!(screen.animation_frame, 0);
    }

    #[test]
    fn test_animation_phase_transitions() {
        let mut screen = BootScreen::new((80, 24));

        // Test Pulsing phase
        screen.update_animation_phase(Duration::from_millis(1000));
        assert_eq!(screen.animation_phase, AnimationPhase::Pulsing);

        // Test Complete phase
        screen.update_animation_phase(Duration::from_millis(2100));
        assert_eq!(screen.animation_phase, AnimationPhase::Complete);
    }
}
