//! Header rendering for LazyLora TUI
//!
//! Renders the application header and search bar as separate bordered sections:
//! - Header: Logo, Live indicator, Network status
//! - Search Bar: Full-width inline search input

use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style, Stylize},
    symbols::border,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::state::App;
use crate::state::ui_state::SearchType;
use crate::theme::{BORDER_STYLE, FOCUSED_BORDER_STYLE, MUTED_COLOR, PRIMARY_COLOR, SUCCESS_COLOR};

use super::helpers::create_border_block;

// ============================================================================
// Header (Logo + Live + Network)
// ============================================================================

/// Render the application header (logo, live indicator, network)
pub fn render_header(frame: &mut Frame, area: Rect, app: &App) {
    let header_block = create_border_block("", false);
    frame.render_widget(header_block.clone(), area);

    if area.height < 3 {
        return;
    }

    let row_y = area.y + 1;

    // Logo (left side)
    let title = if app.show_live {
        create_animated_logo(app.animation_tick)
    } else {
        create_static_logo()
    };

    let title_paragraph = Paragraph::new(title)
        .style(Style::default())
        .alignment(Alignment::Left);

    let title_area = Rect::new(area.x + 2, row_y, 12.min(area.width.saturating_sub(2)), 1);
    frame.render_widget(title_paragraph, title_area);

    // Live indicator (after logo)
    let live_indicator = create_live_indicator(app.show_live);
    let live_area = Rect::new(area.x + 14, row_y, 10, 1);
    frame.render_widget(Paragraph::new(live_indicator), live_area);

    // Network indicator (right side)
    if area.width > 40 {
        render_network_indicator(frame, area, row_y, app);
    }
}

// ============================================================================
// Search Bar (Separate Bordered Section)
// ============================================================================

/// Render the search bar as a separate bordered section
pub fn render_search_bar(frame: &mut Frame, area: Rect, app: &App) {
    // Use focused border style when search is focused
    let search_block = if app.ui.search_focused {
        Block::default()
            .borders(Borders::ALL)
            .border_set(border::ROUNDED)
            .border_style(FOCUSED_BORDER_STYLE)
    } else {
        Block::default()
            .borders(Borders::ALL)
            .border_set(border::ROUNDED)
            .border_style(BORDER_STYLE)
    };
    frame.render_widget(search_block, area);

    if area.height < 3 {
        return;
    }

    let row_y = area.y + 1;
    let search_line = build_search_line(app);
    let search_area = Rect::new(area.x + 2, row_y, area.width.saturating_sub(4), 1);
    frame.render_widget(Paragraph::new(search_line), search_area);
}

/// Build the search bar line based on current state
fn build_search_line(app: &App) -> Line<'static> {
    let ui = &app.ui;

    if ui.search_loading {
        // Show loading state
        return Line::from(vec![
            Span::styled("[/] ", Style::default().fg(MUTED_COLOR)),
            Span::styled("Searching...", Style::default().fg(MUTED_COLOR)),
        ]);
    }

    if ui.search_focused {
        // Focused: show input with cursor and type indicator
        build_focused_search_line(app)
    } else if ui.search_input.is_empty() {
        // Unfocused, empty: show placeholder
        Line::from(vec![
            Span::styled("[/] ", Style::default().fg(MUTED_COLOR)),
            Span::styled("Search...", Style::default().fg(MUTED_COLOR)),
        ])
    } else {
        // Unfocused, has content: show last search
        let type_indicator = match ui.get_effective_search_type() {
            Some(SearchType::Transaction) => "T",
            Some(SearchType::Block) => "B",
            Some(SearchType::Account) => "A",
            Some(SearchType::Asset) => "$",
            None => "?",
        };

        Line::from(vec![
            Span::styled("[/] ", Style::default().fg(MUTED_COLOR)),
            Span::styled(
                format!("[{}] ", type_indicator),
                Style::default().fg(MUTED_COLOR),
            ),
            Span::styled(
                truncate_search_display(&ui.search_input, 40),
                Style::default().fg(MUTED_COLOR),
            ),
        ])
    }
}

/// Build the focused search line with cursor at correct position
fn build_focused_search_line(app: &App) -> Line<'static> {
    let ui = &app.ui;
    let query = &ui.search_input;
    let cursor_pos = ui.cursor_position();

    // Type indicator badge
    let type_char = match ui.get_effective_search_type() {
        Some(SearchType::Transaction) => "T",
        Some(SearchType::Block) => "B",
        Some(SearchType::Account) => "A",
        Some(SearchType::Asset) => "$",
        None => "?",
    };

    let type_color = match ui.get_effective_search_type() {
        Some(SearchType::Transaction) => Color::Cyan,
        Some(SearchType::Block) => Color::Yellow,
        Some(SearchType::Account) => Color::Magenta,
        Some(SearchType::Asset) => Color::Green,
        None => Color::Gray,
    };

    let mut spans = vec![
        Span::styled("[/] ", Style::default().fg(PRIMARY_COLOR)),
        Span::styled(
            format!("[{type_char}]"),
            Style::default().fg(type_color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" ", Style::default()),
    ];

    // Query text with cursor at correct position
    if query.is_empty() {
        // Cursor then placeholder
        spans.push(Span::styled("▌", Style::default().fg(PRIMARY_COLOR)));
        spans.push(Span::styled(
            "Type to search...",
            Style::default().fg(MUTED_COLOR),
        ));
    } else {
        // Split text at cursor position (handle char boundaries)
        let (before, after) = split_at_char_boundary(query, cursor_pos);

        if !before.is_empty() {
            spans.push(Span::styled(
                before,
                Style::default().add_modifier(Modifier::BOLD),
            ));
        }

        spans.push(Span::styled("▌", Style::default().fg(PRIMARY_COLOR)));

        if !after.is_empty() {
            spans.push(Span::styled(
                after,
                Style::default().add_modifier(Modifier::BOLD),
            ));
        }
    }

    Line::from(spans)
}

/// Split string at byte position, ensuring we don't split in middle of a char
fn split_at_char_boundary(s: &str, byte_pos: usize) -> (String, String) {
    // Clamp to valid range
    let pos = byte_pos.min(s.len());

    // Find nearest char boundary
    let mut boundary = pos;
    while boundary > 0 && !s.is_char_boundary(boundary) {
        boundary -= 1;
    }

    (s[..boundary].to_string(), s[boundary..].to_string())
}

/// Truncate search input for display
fn truncate_search_display(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Create the live indicator span
fn create_live_indicator(is_live: bool) -> Line<'static> {
    if is_live {
        Line::from(vec![
            Span::styled("● ", Style::default().fg(SUCCESS_COLOR)),
            Span::styled(
                "LIVE",
                Style::default()
                    .fg(SUCCESS_COLOR)
                    .add_modifier(Modifier::BOLD),
            ),
        ])
    } else {
        Line::from(vec![
            Span::styled("○ ", Style::default().fg(MUTED_COLOR)),
            Span::styled("PAUSED", Style::default().fg(MUTED_COLOR)),
        ])
    }
}

/// Create the animated logo with shimmer effect
fn create_animated_logo(animation_tick: u64) -> Line<'static> {
    let time = animation_tick as f32 * 0.15;

    let bracket_glow = ((time * 0.8).sin() + 1.0) / 2.0;
    let lazy_glow = ((time * 0.8 + 0.5).sin() + 1.0) / 2.0;
    let lora_glow = ((time * 0.8 + 1.0).sin() + 1.0) / 2.0;

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
fn render_network_indicator(frame: &mut Frame, area: Rect, row_y: u16, app: &App) {
    let network_text = app.network.as_str();
    let network_style = Style::default()
        .fg(SUCCESS_COLOR)
        .add_modifier(Modifier::BOLD);

    let network_label = Paragraph::new(network_text)
        .style(network_style)
        .alignment(Alignment::Right);

    let network_area = Rect::new(area.right() - 12, row_y, 10, 1);
    frame.render_widget(network_label, network_area);
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Network;
    use crate::state::{App, StartupOptions};
    use ratatui::{Terminal, backend::TestBackend};
    use rstest::*;

    #[fixture]
    fn test_terminal() -> Terminal<TestBackend> {
        Terminal::new(TestBackend::new(120, 3)).expect("terminal should be created")
    }

    fn create_test_app() -> App {
        let rt = tokio::runtime::Runtime::new().expect("runtime");
        rt.block_on(async {
            App::new(StartupOptions {
                network: Some(Network::TestNet),
                search: None,
                graph_view: false,
            })
            .await
            .expect("app")
        })
    }

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

    #[rstest]
    fn test_header_renders(test_terminal: Terminal<TestBackend>) {
        let mut terminal = test_terminal;
        let app = create_test_app();

        terminal
            .draw(|frame| render_header(frame, frame.area(), &app))
            .expect("draw");

        // Just verify it renders without panic
        let backend = terminal.backend();
        let buffer = backend.buffer();
        // Check logo is present
        assert!(buffer.content().iter().any(|c| c.symbol() == "["));
    }

    #[rstest]
    fn test_search_bar_renders(test_terminal: Terminal<TestBackend>) {
        let mut terminal = test_terminal;
        let app = create_test_app();

        terminal
            .draw(|frame| render_search_bar(frame, frame.area(), &app))
            .expect("draw");

        // Verify search placeholder is present
        let backend = terminal.backend();
        let buffer = backend.buffer();
        let content: String = buffer.content().iter().map(|c| c.symbol()).collect();
        assert!(content.contains("Search"));
    }

    #[test]
    fn test_build_search_line_unfocused_empty() {
        let app = create_test_app();
        let line = build_search_line(&app);
        // Should show placeholder
        let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(text.contains("Search"));
    }

    #[test]
    fn test_build_search_line_focused() {
        let mut app = create_test_app();
        app.ui.focus_search();
        app.ui.search_type_char('t');
        app.ui.search_type_char('e');
        app.ui.search_type_char('s');
        app.ui.search_type_char('t');

        let line = build_search_line(&app);
        let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(text.contains("test"));
        assert!(text.contains("▌")); // cursor
    }

    #[test]
    fn test_truncate_search_display() {
        assert_eq!(truncate_search_display("short", 10), "short");
        assert_eq!(
            truncate_search_display("this is a long string", 10),
            "this is..."
        );
    }
}
