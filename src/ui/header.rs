//! Header rendering for LazyLora TUI
//!
//! Renders the application header with logo, network status, and live indicator.

use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::state::App;
use crate::theme::SUCCESS_COLOR;

use super::helpers::create_border_block;

/// Render the application header
pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let header_block = create_border_block("", false);
    frame.render_widget(header_block.clone(), area);

    if area.height <= 2 {
        return;
    }

    // Create the title with shimmer effect when live mode is enabled
    let title = if app.show_live {
        create_animated_logo(app.animation_tick)
    } else {
        create_static_logo()
    };

    let title_paragraph = Paragraph::new(title)
        .style(Style::default())
        .alignment(Alignment::Left);

    let title_area = Rect::new(
        area.x + 2,
        area.y + 1,
        12.min(area.width.saturating_sub(2)),
        1,
    );
    frame.render_widget(title_paragraph, title_area);

    // Render network indicator on the right
    if area.width > 40 {
        render_network_indicator(frame, area, app);
    }
}

/// Create the animated logo with shimmer effect
fn create_animated_logo(animation_tick: u64) -> Line<'static> {
    // Calculate shimmer effect using sine wave for breathing glow
    let time = animation_tick as f32 * 0.15;

    // Create phase-shifted sine waves for different parts of the logo
    let bracket_glow = ((time * 0.8).sin() + 1.0) / 2.0;
    let lazy_glow = ((time * 0.8 + 0.5).sin() + 1.0) / 2.0;
    let lora_glow = ((time * 0.8 + 1.0).sin() + 1.0) / 2.0;

    // Map glow values to color intensity
    let lazy_green = (120.0 + lazy_glow * 135.0) as u8;
    let lazy_color = Color::Rgb(
        (50.0 * lazy_glow) as u8,
        lazy_green,
        (80.0 * lazy_glow) as u8,
    );

    let lora_blue = (140.0 + lora_glow * 115.0) as u8;
    let lora_green = (180.0 + lora_glow * 75.0) as u8;
    let lora_color = Color::Rgb((100.0 * lora_glow) as u8, lora_green, lora_blue);

    let bracket_intensity = (100.0 + bracket_glow * 155.0) as u8;
    let bracket_color = Color::Rgb(bracket_intensity, bracket_intensity, bracket_intensity);

    Line::from(vec![
        Span::styled("[", Style::default().fg(bracket_color)),
        Span::styled(
            "lazy",
            Style::default().fg(lazy_color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "lora",
            Style::default().fg(lora_color).add_modifier(Modifier::BOLD),
        ),
        Span::styled("]", Style::default().fg(bracket_color)),
    ])
}

/// Create the static logo (when live mode is off)
fn create_static_logo() -> Line<'static> {
    Line::from(vec![
        "[".into(),
        "lazy".green().bold(),
        "lora".blue().bold(),
        "]".into(),
    ])
}

/// Render the network indicator on the right side
fn render_network_indicator(frame: &mut Frame, area: Rect, app: &App) {
    let network_text = format!("Network: {}", app.network.as_str());
    let network_style = Style::default()
        .fg(SUCCESS_COLOR)
        .add_modifier(Modifier::BOLD);

    let network_label = Paragraph::new(network_text)
        .style(network_style)
        .alignment(Alignment::Right);

    let network_area = Rect::new(area.right() - 20, area.y + 1, 18, 1);
    frame.render_widget(network_label, network_area);
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_static_logo() {
        let logo = create_static_logo();
        assert_eq!(logo.spans.len(), 4);
    }

    #[test]
    fn test_create_animated_logo() {
        let logo = create_animated_logo(0);
        assert_eq!(logo.spans.len(), 4);

        // Test with different tick values
        let logo2 = create_animated_logo(100);
        assert_eq!(logo2.spans.len(), 4);
    }
}
