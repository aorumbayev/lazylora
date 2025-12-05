//! Boot screen module for LazyLora TUI application.
//!
//! This module provides an animated boot screen with fade-in effects,
//! breathing glow animations, and responsive scaling for different terminal sizes.
//!
//! # Features
//! - Fade-in effect from center outward
//! - Breathing glow effect using sine wave
//! - Adaptive scaling for different terminal sizes (full, medium, compact)
//! - Skip functionality with any key (Ctrl+C exits application)
//! - TTY detection to skip boot screen in non-interactive environments
//!
//! # Example
//! ```rust,ignore
//! use lazylora::boot_screen::BootScreen;
//!
//! let mut boot_screen = BootScreen::new((80, 24));
//! let should_continue = boot_screen.run(|screen, frame| {
//!     screen.draw(frame);
//! }).await?;
//! ```

use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, poll, read};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, Paragraph, Wrap},
};
use std::time::{Duration, Instant};
use tokio::time::sleep;

/// Boot screen animation phases
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnimationPhase {
    /// Initial fade-in effect with pulsing dots
    FadeIn,
    /// Main logo display with breathing glow
    ShowLogo,
    /// Loading indicator phase
    Loading,
    /// Animation complete, ready to proceed
    Complete,
}

/// Boot screen configuration and state for LazyLora TUI.
///
/// Manages the animated boot sequence including fade-in effects,
/// logo display with breathing glow, and loading indicators.
pub struct BootScreen {
    /// Time when the boot screen started
    start_time: Instant,
    /// Current animation phase
    animation_phase: AnimationPhase,
    /// Loading dots animation counter (0-3)
    show_loading_dots: u8,
    /// Frame counter for smooth animations
    animation_frame: u32,
    /// Terminal dimensions for responsive scaling
    #[allow(dead_code)]
    terminal_size: (u16, u16),
}

impl BootScreen {
    /// Create a new boot screen with the given terminal dimensions.
    ///
    /// # Arguments
    /// * `terminal_size` - Tuple of (width, height) in terminal cells
    ///
    /// # Example
    /// ```rust,ignore
    /// let boot_screen = BootScreen::new((120, 40));
    /// ```
    pub fn new(terminal_size: (u16, u16)) -> Self {
        Self {
            start_time: Instant::now(),
            animation_phase: AnimationPhase::FadeIn,
            show_loading_dots: 0,
            animation_frame: 0,
            terminal_size,
        }
    }

    /// Run the boot screen animation.
    ///
    /// This method handles the complete boot screen lifecycle including:
    /// - Terminal setup and cleanup
    /// - Animation frame rendering at ~30 FPS
    /// - User input handling (skip with any key, exit with Ctrl+C)
    /// - TTY detection to skip in non-interactive environments
    ///
    /// # Arguments
    /// * `draw_fn` - Closure that draws the boot screen frame
    ///
    /// # Returns
    /// * `Ok(true)` - Continue to main application
    /// * `Ok(false)` - Exit application (user pressed Ctrl+C)
    /// * `Err(_)` - Terminal error occurred
    ///
    /// # Example
    /// ```rust,ignore
    /// let should_continue = boot_screen.run(|screen, frame| {
    ///     screen.draw(frame);
    /// }).await?;
    ///
    /// if !should_continue {
    ///     return Ok(()); // User requested exit
    /// }
    /// ```
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
        let boot_duration = Duration::from_millis(2200); // ~2.2 seconds total
        let frame_duration = Duration::from_millis(33); // ~30 FPS for smooth animation

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
        self.animation_phase = match elapsed.as_millis() {
            0..=400 => AnimationPhase::FadeIn,      // 0.4s fade-in
            401..=1600 => AnimationPhase::ShowLogo, // 1.2s logo display
            1601..=2000 => AnimationPhase::Loading, // 0.4s loading
            _ => AnimationPhase::Complete,
        };

        // Update loading dots animation (cycle every 150ms)
        if elapsed.as_millis().is_multiple_of(150) {
            self.show_loading_dots = (self.show_loading_dots + 1) % 4;
        }
    }

    /// Cubic ease-in-out function for smooth acceleration and deceleration.
    ///
    /// # Arguments
    /// * `t` - Progress value from 0.0 to 1.0
    ///
    /// # Returns
    /// Eased value from 0.0 to 1.0
    fn ease_in_out_cubic(t: f32) -> f32 {
        if t < 0.5 {
            4.0 * t * t * t
        } else {
            1.0 - (-2.0 * t + 2.0).powf(3.0) / 2.0
        }
    }

    /// Cubic ease-out function for smooth deceleration.
    ///
    /// # Arguments
    /// * `t` - Progress value from 0.0 to 1.0
    ///
    /// # Returns
    /// Eased value from 0.0 to 1.0
    fn ease_out_cubic(t: f32) -> f32 {
        1.0 - (1.0 - t).powf(3.0)
    }

    /// Apply fade-in animation with breathing glow effect to ASCII art lines.
    ///
    /// Creates a staggered fade-in effect from center outward with a subtle
    /// sine wave-based breathing glow that adds visual interest.
    fn apply_animation_effects(
        &self,
        lines: Vec<Line<'static>>,
        progress: f32,
    ) -> Vec<Line<'static>> {
        // Breathing glow using sine wave: oscillates between 0.7 and 1.0
        let time = self.animation_frame as f32 * 0.05;
        let glow = (time.sin() + 1.0) / 2.0 * 0.3 + 0.7;

        let total_lines = lines.len();
        let center = total_lines / 2;

        lines
            .into_iter()
            .enumerate()
            .map(|(i, line)| {
                // Calculate staggered fade-in from center outward
                let distance_from_center =
                    ((i as i32 - center as i32).abs() as f32) / (total_lines as f32);
                let line_progress = ((progress - distance_from_center * 0.3) * 1.5).clamp(0.0, 1.0);

                // Apply fade and glow to each span
                let faded_spans = line
                    .spans
                    .into_iter()
                    .map(|span| {
                        let opacity = (line_progress * glow * 255.0) as u8;

                        // Determine color based on original style with ANSI colors for theme adaptation
                        let new_color = match span.style.fg {
                            // Green text (LAZY part)
                            Some(Color::Green) | Some(Color::Indexed(10)) => {
                                if opacity > 128 {
                                    Color::Indexed(10) // ANSI bright green
                                } else {
                                    Color::Indexed(2) // ANSI regular green (dimmed)
                                }
                            }
                            // Blue/Cyan text (LORA part)
                            Some(Color::Blue)
                            | Some(Color::Cyan)
                            | Some(Color::Indexed(12))
                            | Some(Color::Indexed(14)) => {
                                if opacity > 128 {
                                    Color::Indexed(14) // ANSI bright cyan
                                } else {
                                    Color::Indexed(6) // ANSI regular cyan (dimmed)
                                }
                            }
                            // Default/white text
                            _ => {
                                if opacity > 200 {
                                    Color::Indexed(15) // ANSI bright white
                                } else if opacity > 100 {
                                    Color::Indexed(7) // ANSI white
                                } else {
                                    Color::Indexed(8) // ANSI bright black (dim)
                                }
                            }
                        };

                        // Add bold modifier when fully visible and glowing
                        let mut style = Style::default().fg(new_color);
                        if line_progress > 0.9 && glow > 0.95 {
                            style = style.add_modifier(Modifier::BOLD);
                        }

                        Span::styled(span.content, style)
                    })
                    .collect::<Vec<_>>();

                Line::from(faded_spans)
            })
            .collect()
    }

    /// Draw the boot screen based on current animation phase.
    ///
    /// # Arguments
    /// * `frame` - Ratatui frame to render to
    pub fn draw(&self, frame: &mut Frame) {
        let area = frame.area();

        // Clear background
        frame.render_widget(Clear, area);

        // Calculate scale factor for responsive layout
        let scale_factor = self.calculate_scale_factor(area.width, area.height);

        match self.animation_phase {
            AnimationPhase::FadeIn => self.draw_fade_in(frame, area),
            AnimationPhase::ShowLogo => self.draw_logo(frame, area, scale_factor),
            AnimationPhase::Loading => self.draw_loading(frame, area, scale_factor),
            AnimationPhase::Complete => self.draw_complete(frame, area, scale_factor),
        }
    }

    /// Calculate scale factor based on terminal dimensions.
    ///
    /// # Returns
    /// Scale factor clamped between 0.1 and 1.5
    fn calculate_scale_factor(&self, width: u16, height: u16) -> f32 {
        // Full logo is ~65 chars wide and 6 lines tall
        let original_width = 65.0;
        let original_height = 8.0;

        let width_scale = if width < 40 {
            (width as f32 * 0.95) / 25.0 // Compact art width
        } else {
            (width as f32 * 0.85) / original_width
        };

        let height_scale = if height < 10 {
            (height as f32 * 0.9) / 4.0 // Compact art height
        } else {
            (height as f32 * 0.6) / original_height
        };

        let min_scale = if width < 30 || height < 8 { 0.1 } else { 0.3 };
        width_scale.min(height_scale).clamp(min_scale, 1.5)
    }

    /// Draw the fade-in phase with pulsing initialization text.
    fn draw_fade_in(&self, frame: &mut Frame, area: Rect) {
        let elapsed_ms = self.start_time.elapsed().as_millis() as f32;
        let fade_progress = Self::ease_in_out_cubic((elapsed_ms / 400.0).min(1.0));

        // Pulsing dot animation
        let pulse_phase = (elapsed_ms / 150.0).sin();
        let dots = if pulse_phase > 0.5 {
            "●"
        } else if pulse_phase > 0.0 {
            "◐"
        } else if pulse_phase > -0.5 {
            "◑"
        } else {
            "◒"
        };

        let center_y = area.height / 2;
        let center_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(center_y),
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .split(area);

        // Create styled initialization text with fade effect
        let fade_text = Paragraph::new(Line::from(vec![
            Span::styled(
                format!("  {dots}  Initializing "),
                Style::default()
                    .fg(Color::Rgb(
                        (255.0 * fade_progress) as u8,
                        (255.0 * fade_progress) as u8,
                        (255.0 * fade_progress) as u8,
                    ))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Lazy",
                Style::default()
                    .fg(Color::Rgb(
                        (100.0 * fade_progress) as u8,
                        (255.0 * fade_progress) as u8,
                        (100.0 * fade_progress) as u8,
                    ))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Lora",
                Style::default()
                    .fg(Color::Rgb(
                        (100.0 * fade_progress) as u8,
                        (200.0 * fade_progress) as u8,
                        (255.0 * fade_progress) as u8,
                    ))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("...  {dots}  "),
                Style::default()
                    .fg(Color::Rgb(
                        (255.0 * fade_progress) as u8,
                        (255.0 * fade_progress) as u8,
                        (255.0 * fade_progress) as u8,
                    ))
                    .add_modifier(Modifier::BOLD),
            ),
        ]))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

        frame.render_widget(fade_text, center_area[1]);
    }

    /// Draw the main logo phase with breathing glow effect.
    fn draw_logo(&self, frame: &mut Frame, area: Rect, _scale_factor: f32) {
        let elapsed_ms = self.start_time.elapsed().as_millis() as f32;
        let raw_progress = ((elapsed_ms - 400.0) / 1200.0).clamp(0.0, 1.0);
        let show_progress = Self::ease_out_cubic(raw_progress);

        // Adaptive layout constraints
        let vertical_constraints = if area.height < 15 {
            let available_height = area.height.saturating_sub(4);
            let margin = available_height / 2;
            [
                Constraint::Length(margin),
                Constraint::Length(4),
                Constraint::Length(margin),
            ]
        } else {
            [
                Constraint::Percentage(25),
                Constraint::Min(8),
                Constraint::Percentage(25),
            ]
        };

        let horizontal_constraints = if area.width < 70 {
            let ascii_width = if area.width < 40 { 20 } else { 45 };
            let available_width = area.width.saturating_sub(ascii_width);
            let margin = available_width / 2;
            [
                Constraint::Length(margin),
                Constraint::Length(ascii_width),
                Constraint::Length(margin),
            ]
        } else {
            [
                Constraint::Percentage(10),
                Constraint::Min(65),
                Constraint::Percentage(10),
            ]
        };

        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vertical_constraints)
            .split(area);

        let horizontal_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(horizontal_constraints)
            .split(vertical_chunks[1]);

        let ascii_area = horizontal_chunks[1];

        // Select ASCII art based on terminal size
        // Use plain text for smaller screens, full ASCII art for large screens
        let base_ascii_lines = if area.width < 70 || area.height < 15 {
            self.get_plain_text_logo()
        } else {
            self.get_full_ascii_art()
        };

        // Apply animation effects
        let animated_lines = self.apply_animation_effects(base_ascii_lines, show_progress);

        let ascii_paragraph = Paragraph::new(animated_lines)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        frame.render_widget(ascii_paragraph, ascii_area);
    }

    /// Draw the loading phase with animated dots.
    fn draw_loading(&self, frame: &mut Frame, area: Rect, scale_factor: f32) {
        // Draw the complete logo first
        self.draw_logo_complete(frame, area, scale_factor);

        // Add loading animation below
        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(55),
                Constraint::Length(3),
                Constraint::Percentage(42),
            ])
            .split(area);

        let loading_dots = match self.show_loading_dots {
            0 => "●  ○  ○",
            1 => "○  ●  ○",
            2 => "○  ○  ●",
            _ => "○  ●  ○",
        };

        let loading_text = Paragraph::new(vec![
            Line::from(vec![Span::styled(
                "Loading blockchain explorer",
                Style::default()
                    .fg(Color::Indexed(14)) // Cyan
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![Span::styled(
                loading_dots,
                Style::default().fg(Color::Indexed(10)), // Green
            )]),
        ])
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

        frame.render_widget(loading_text, vertical_chunks[1]);

        // Add skip instruction at bottom
        let skip_text = Paragraph::new(Line::from(vec![Span::styled(
            "Press any key to skip",
            Style::default()
                .fg(Color::Indexed(8)) // Dim gray
                .add_modifier(Modifier::ITALIC),
        )]))
        .alignment(Alignment::Center);

        let bottom_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .split(vertical_chunks[2]);

        frame.render_widget(skip_text, bottom_area[1]);
    }

    /// Draw the complete phase with "Ready!" message.
    fn draw_complete(&self, frame: &mut Frame, area: Rect, scale_factor: f32) {
        self.draw_logo_complete(frame, area, scale_factor);

        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(60),
                Constraint::Length(1),
                Constraint::Percentage(39),
            ])
            .split(area);

        let ready_text = Paragraph::new(Line::from(vec![Span::styled(
            "✨ Ready! ✨",
            Style::default()
                .fg(Color::Indexed(15)) // Bright white
                .add_modifier(Modifier::BOLD),
        )]))
        .alignment(Alignment::Center);

        frame.render_widget(ready_text, vertical_chunks[1]);
    }

    /// Draw the logo at full visibility (no animation effects).
    fn draw_logo_complete(&self, frame: &mut Frame, area: Rect, _scale_factor: f32) {
        let vertical_constraints = if area.height < 15 {
            let available_height = area.height.saturating_sub(4);
            let margin = available_height / 2;
            [
                Constraint::Length(margin),
                Constraint::Length(4),
                Constraint::Length(margin),
            ]
        } else {
            [
                Constraint::Percentage(20),
                Constraint::Min(8),
                Constraint::Percentage(30),
            ]
        };

        let horizontal_constraints = if area.width < 70 {
            let ascii_width = if area.width < 40 { 20 } else { 45 };
            let available_width = area.width.saturating_sub(ascii_width);
            let margin = available_width / 2;
            [
                Constraint::Length(margin),
                Constraint::Length(ascii_width),
                Constraint::Length(margin),
            ]
        } else {
            [
                Constraint::Percentage(10),
                Constraint::Min(65),
                Constraint::Percentage(10),
            ]
        };

        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vertical_constraints)
            .split(area);

        let horizontal_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(horizontal_constraints)
            .split(vertical_chunks[1]);

        let ascii_area = horizontal_chunks[1];

        // Select ASCII art based on terminal size
        // Use plain text for smaller screens, full ASCII art for large screens
        let ascii_lines = if area.width < 70 || area.height < 15 {
            self.get_plain_text_logo()
        } else {
            self.get_full_ascii_art()
        };

        let ascii_paragraph = Paragraph::new(ascii_lines)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        frame.render_widget(ascii_paragraph, ascii_area);
    }

    /// Get full-size ASCII art with "LAZY" in green and "LORA" in blue.
    fn get_full_ascii_art(&self) -> Vec<Line<'static>> {
        // Split the logo into LAZY (green) and LORA (blue) parts
        // Each line is split at approximately the midpoint
        vec![
            // Line 1
            Line::from(vec![
                Span::styled(
                    "██╗      █████╗ ███████╗██╗   ██╗",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    " ██╗      ██████╗ ██████╗  █████╗",
                    Style::default()
                        .fg(Color::Indexed(14)) // Cyan
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            // Line 2
            Line::from(vec![
                Span::styled(
                    "██║     ██╔══██╗╚══███╔╝╚██╗ ██╔╝",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "██║     ██╔═══██╗██╔══██╗██╔══██╗",
                    Style::default()
                        .fg(Color::Indexed(14))
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            // Line 3
            Line::from(vec![
                Span::styled(
                    "██║     ███████║  ███╔╝  ╚████╔╝ ",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "██║     ██║   ██║██████╔╝███████║",
                    Style::default()
                        .fg(Color::Indexed(14))
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            // Line 4
            Line::from(vec![
                Span::styled(
                    "██║     ██╔══██║ ███╔╝    ╚██╔╝  ",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "██║     ██║   ██║██╔══██╗██╔══██║",
                    Style::default()
                        .fg(Color::Indexed(14))
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            // Line 5
            Line::from(vec![
                Span::styled(
                    "███████╗██║  ██║███████╗   ██║   ",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "███████╗╚██████╔╝██║  ██║██║  ██║",
                    Style::default()
                        .fg(Color::Indexed(14))
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            // Line 6
            Line::from(vec![
                Span::styled(
                    "╚══════╝╚═╝  ╚═╝╚══════╝   ╚═╝   ",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "╚══════╝ ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝",
                    Style::default()
                        .fg(Color::Indexed(14))
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
        ]
    }

    /// Get plain text logo for small screens.
    fn get_plain_text_logo(&self) -> Vec<Line<'static>> {
        vec![Line::from(vec![
            Span::styled(
                "Lazy",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Lora",
                Style::default()
                    .fg(Color::Indexed(14))
                    .add_modifier(Modifier::BOLD),
            ),
        ])]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boot_screen_creation() {
        let screen = BootScreen::new((80, 24));
        assert_eq!(screen.animation_phase, AnimationPhase::FadeIn);
        assert_eq!(screen.show_loading_dots, 0);
        assert_eq!(screen.animation_frame, 0);
    }

    #[test]
    fn test_animation_phase_transitions() {
        let mut screen = BootScreen::new((80, 24));

        // Test FadeIn phase
        screen.update_animation_phase(Duration::from_millis(200));
        assert_eq!(screen.animation_phase, AnimationPhase::FadeIn);

        // Test ShowLogo phase
        screen.update_animation_phase(Duration::from_millis(600));
        assert_eq!(screen.animation_phase, AnimationPhase::ShowLogo);

        // Test Loading phase
        screen.update_animation_phase(Duration::from_millis(1700));
        assert_eq!(screen.animation_phase, AnimationPhase::Loading);

        // Test Complete phase
        screen.update_animation_phase(Duration::from_millis(2100));
        assert_eq!(screen.animation_phase, AnimationPhase::Complete);
    }

    #[test]
    fn test_easing_functions() {
        // Test ease_in_out_cubic boundaries
        assert!((BootScreen::ease_in_out_cubic(0.0) - 0.0).abs() < f32::EPSILON);
        assert!((BootScreen::ease_in_out_cubic(1.0) - 1.0).abs() < f32::EPSILON);

        // Test ease_out_cubic boundaries
        assert!((BootScreen::ease_out_cubic(0.0) - 0.0).abs() < f32::EPSILON);
        assert!((BootScreen::ease_out_cubic(1.0) - 1.0).abs() < f32::EPSILON);

        // Test midpoint behavior
        let mid_in_out = BootScreen::ease_in_out_cubic(0.5);
        assert!(mid_in_out > 0.4 && mid_in_out < 0.6);
    }

    #[test]
    fn test_scale_factor_calculation() {
        let screen = BootScreen::new((80, 24));

        // Large terminal
        let large_scale = screen.calculate_scale_factor(120, 40);
        assert!(large_scale >= 1.0);

        // Small terminal
        let small_scale = screen.calculate_scale_factor(40, 12);
        assert!(small_scale < 1.0);

        // Very small terminal
        let tiny_scale = screen.calculate_scale_factor(20, 6);
        assert!(tiny_scale >= 0.1);
    }
}
