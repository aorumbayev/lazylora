//! Footer rendering module for the LazyLora TUI.
//!
//! This module provides the footer bar that displays keyboard shortcuts
//! and other contextual hints at the bottom of the screen.

use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::Style,
    widgets::Paragraph,
};

use crate::state::App;
use crate::theme::MUTED_COLOR;

// ============================================================================
// Footer Rendering
// ============================================================================

/// Renders the footer bar with keyboard shortcuts.
///
/// The footer displays common keybindings available to the user based on
/// the current application context. This helps with discoverability of features.
///
/// # Arguments
///
/// * `frame` - The ratatui frame to render to
/// * `area` - The rectangular area to render the footer in
/// * `_app` - The application state (currently unused but available for future context-sensitive hints)
///
/// # Future Enhancements
///
/// TODO: Show context-sensitive hints based on current view:
/// - Different shortcuts when viewing block details
/// - Different shortcuts when viewing transaction details
/// - Different shortcuts when search popup is open
/// - Highlight the most relevant shortcuts for current context
///
/// # Example
///
/// ```ignore
/// use ratatui::Frame;
/// use crate::ui::footer;
/// use crate::state::App;
///
/// fn render_ui(app: &App, frame: &mut Frame) {
///     let footer_area = /* calculate footer area */;
///     footer::render(frame, footer_area, app);
/// }
/// ```
pub fn render(frame: &mut Frame, area: Rect, _app: &App) {
    let footer_text = "q:Quit  r:Refresh  f:Search  n:Network  Space:Live  Tab:Focus";
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(MUTED_COLOR))
        .alignment(Alignment::Center);

    frame.render_widget(footer, area);
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    #[test]
    fn test_render_footer_displays_shortcuts() {
        // Use a simple mock instead of full App construction
        test_with_mock_app(|app| {
            let backend = TestBackend::new(80, 1);
            let mut terminal = Terminal::new(backend).unwrap();

            terminal
                .draw(|frame| {
                    let area = frame.area();
                    render(frame, area, app);
                })
                .unwrap();

            let buffer = terminal.backend().buffer();
            let content = buffer_to_string(buffer, 80, 1);

            // Verify key shortcuts are present
            assert!(content.contains("q:Quit"));
            assert!(content.contains("r:Refresh"));
            assert!(content.contains("f:Search"));
            assert!(content.contains("n:Network"));
            assert!(content.contains("Space:Live"));
            assert!(content.contains("Tab:Focus"));
        });
    }

    #[test]
    fn test_render_footer_handles_narrow_width() {
        test_with_mock_app(|app| {
            let backend = TestBackend::new(40, 1);
            let mut terminal = Terminal::new(backend).unwrap();

            terminal
                .draw(|frame| {
                    let area = frame.area();
                    render(frame, area, app);
                })
                .unwrap();

            // Should not panic with narrow width
            let buffer = terminal.backend().buffer();
            assert!(buffer.area().width == 40);
        });
    }

    #[test]
    fn test_render_footer_handles_zero_height() {
        test_with_mock_app(|app| {
            let backend = TestBackend::new(80, 1);
            let mut terminal = Terminal::new(backend).unwrap();

            terminal
                .draw(|frame| {
                    let area = Rect::new(0, 0, 80, 0);
                    render(frame, area, app);
                })
                .unwrap();

            // Should not panic with zero height
        });
    }

    #[test]
    fn test_render_footer_text_centered() {
        test_with_mock_app(|app| {
            let backend = TestBackend::new(80, 1);
            let mut terminal = Terminal::new(backend).unwrap();

            terminal
                .draw(|frame| {
                    let area = frame.area();
                    render(frame, area, app);
                })
                .unwrap();

            let buffer = terminal.backend().buffer();
            let content = buffer_to_string(buffer, 80, 1);

            // Text should be centered - check that there are spaces on both sides
            let trimmed = content.trim();
            let leading_spaces = content.len() - content.trim_start().len();
            let trailing_spaces = content.len() - content.trim_end().len();

            // For centered text, leading and trailing spaces should be roughly equal
            // (allowing for off-by-one due to odd total width)
            let diff = (leading_spaces as i32 - trailing_spaces as i32).abs();
            assert!(
                diff <= 1,
                "Text should be centered. Leading: {}, Trailing: {}",
                leading_spaces,
                trailing_spaces
            );
            assert!(!trimmed.is_empty(), "Footer should have content");
        });
    }

    // Helper function to run tests with a mock App
    // Since App construction is complex and requires async runtime,
    // we use a helper that creates it properly
    fn test_with_mock_app<F>(test_fn: F)
    where
        F: FnOnce(&App),
    {
        use crate::domain::Network;
        use crate::state::StartupOptions;

        // Create a minimal runtime for App construction
        let rt = tokio::runtime::Runtime::new().unwrap();
        let app = rt.block_on(async {
            let options = StartupOptions {
                network: Some(Network::TestNet),
                search: None,
                graph_view: false,
            };
            App::new(options).await.unwrap()
        });

        test_fn(&app);
    }

    // Helper to convert buffer to string for assertions
    fn buffer_to_string(buffer: &ratatui::buffer::Buffer, width: u16, height: u16) -> String {
        let mut result = String::new();
        for y in 0..height {
            for x in 0..width {
                result.push_str(buffer[(x, y)].symbol());
            }
            if y < height - 1 {
                result.push('\n');
            }
        }
        result
    }
}
