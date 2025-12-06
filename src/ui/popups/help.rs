//! Help popup showing all keybindings organized by context.
//!
//! Displays a scrollable popup with keybinding sections (Global, Navigation,
//! Detail View, Search). Activated by '?' key, closed by Esc/q/?.

use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

use crate::theme::ACCENT_COLOR;
use crate::ui::layout::centered_popup_area;

// ============================================================================
// Keybinding Data (Hardcoded per Commandment 5)
// ============================================================================

/// Hardcoded keybinding sections with descriptions.
const HELP_SECTIONS: &[(&str, &[(&str, &str)])] = &[
    (
        "Global Keys",
        &[
            ("q", "Quit application"),
            ("r", "Refresh data"),
            ("?", "Toggle help"),
            ("n", "Network select"),
            ("Space", "Toggle live updates"),
            ("f", "Focus search"),
        ],
    ),
    (
        "Navigation",
        &[
            ("Tab", "Cycle panels"),
            ("↑ / k", "Move up"),
            ("↓ / j", "Move down"),
            ("g", "Go to top"),
            ("G", "Go to bottom"),
            ("Enter", "Select/Open details"),
            ("Esc", "Close/Cancel"),
        ],
    ),
    (
        "Detail View",
        &[
            ("Esc", "Close details"),
            ("Tab", "Switch view mode"),
            ("c", "Copy ID"),
            ("y", "Copy JSON"),
            ("o", "Open in browser"),
            ("f", "Toggle fullscreen"),
            ("j / k", "Navigate sections (table)"),
            ("Enter", "Toggle section (table)"),
            ("↑↓←→", "Scroll (graph view)"),
            ("s", "Export SVG (graph view)"),
        ],
    ),
    (
        "Search",
        &[
            ("Esc", "Cancel search"),
            ("Enter", "Submit query"),
            ("Tab", "Cycle search type"),
            ("↑ / ↓", "Search history"),
            ("← / →", "Move cursor"),
            ("Backspace", "Delete character"),
        ],
    ),
];

// ============================================================================
// Public API
// ============================================================================

/// Renders the help popup with keybindings organized by section.
///
/// Displays a scrollable list of keybinding sections with keyboard shortcuts
/// and their descriptions. The popup is centered and sized to 70% width, 80% height.
///
/// # Arguments
///
/// * `frame` - The Ratatui frame to render to
/// * `area` - The terminal area to render within
/// * `scroll_offset` - The current scroll position (in lines)
///
/// # Example
///
/// ```ignore
/// use lazylora::ui::popups::help;
///
/// help::render(&mut frame, area, 0);
/// ```
pub fn render(frame: &mut Frame, area: Rect, scroll_offset: u16) {
    // Calculate popup size (70% width, 80% height)
    let width = (area.width * 7 / 10).max(50).min(area.width);
    let height = (area.height * 8 / 10).max(20).min(area.height);
    let popup_area = centered_popup_area(area, width, height);

    // Clear background
    frame.render_widget(Clear, popup_area);

    // Create popup block
    let block = Block::default()
        .title(" Help (? to close) ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ACCENT_COLOR));

    frame.render_widget(block.clone(), popup_area);

    // Calculate inner content area
    let inner = block.inner(popup_area);

    // Build help content with styled sections
    let mut lines = Vec::new();

    for (section_title, bindings) in HELP_SECTIONS {
        // Section title (bold, colored)
        lines.push(Line::from(vec![Span::styled(
            *section_title,
            Style::default()
                .fg(ACCENT_COLOR)
                .add_modifier(Modifier::BOLD),
        )]));

        // Section separator
        lines.push(Line::from(
            "─".repeat(inner.width.saturating_sub(2) as usize),
        ));

        // Keybindings (key in accent color, description in default)
        for (key, description) in *bindings {
            lines.push(Line::from(vec![
                Span::styled(format!("{:<12}", key), Style::default().fg(ACCENT_COLOR)),
                Span::raw(*description),
            ]));
        }

        // Blank line between sections
        lines.push(Line::raw(""));
    }

    // Calculate total scrollable content
    let total_lines = lines.len() as u16;
    let visible_lines = inner.height;

    // Clamp scroll offset to valid range
    let max_scroll = total_lines.saturating_sub(visible_lines);
    let clamped_scroll = scroll_offset.min(max_scroll);

    // Create scrollable paragraph
    let paragraph = Paragraph::new(lines)
        .style(Style::default())
        .wrap(Wrap { trim: false })
        .scroll((clamped_scroll, 0));

    frame.render_widget(paragraph, inner);

    // Render scroll indicator if content is scrollable
    if total_lines > visible_lines {
        render_scroll_indicator(frame, popup_area, clamped_scroll, max_scroll);
    }
}

// ============================================================================
// Internal Helpers
// ============================================================================

/// Renders a scroll indicator at the bottom right of the popup.
fn render_scroll_indicator(frame: &mut Frame, popup_area: Rect, scroll: u16, max_scroll: u16) {
    if max_scroll == 0 {
        return;
    }

    let indicator = if scroll >= max_scroll {
        "━"
    } else if scroll == 0 {
        "┯"
    } else {
        "╂"
    };

    let indicator_area = Rect::new(
        popup_area.x + popup_area.width - 2,
        popup_area.y + popup_area.height - 1,
        1,
        1,
    );

    let indicator_widget = Paragraph::new(indicator)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);

    frame.render_widget(indicator_widget, indicator_area);
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};

    #[test]
    fn test_help_popup_renders_without_scroll() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                render(frame, frame.area(), 0);
            })
            .unwrap();

        // Should render without panicking
        let buffer = terminal.backend().buffer();
        assert!(!buffer.area().is_empty());
    }

    #[test]
    fn test_help_popup_renders_with_scroll() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        // Test various scroll positions
        for scroll in [0, 5, 10, 100] {
            terminal
                .draw(|frame| {
                    render(frame, frame.area(), scroll);
                })
                .unwrap();
        }

        // Should render all scroll positions without panicking
        let buffer = terminal.backend().buffer();
        assert!(!buffer.area().is_empty());
    }

    #[test]
    fn test_help_popup_small_terminal() {
        // Test at minimum viable size (Commandment 22: design for 80×24)
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                render(frame, frame.area(), 0);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        assert!(!buffer.area().is_empty());
    }

    #[test]
    fn test_help_sections_not_empty() {
        // Ensure we have content to display
        assert!(!HELP_SECTIONS.is_empty(), "Help sections must not be empty");

        for (section_name, bindings) in HELP_SECTIONS {
            assert!(!section_name.is_empty(), "Section name must not be empty");
            assert!(!bindings.is_empty(), "Section must have bindings");

            for (key, desc) in *bindings {
                assert!(!key.is_empty(), "Key binding must not be empty");
                assert!(!desc.is_empty(), "Description must not be empty");
            }
        }
    }
}
