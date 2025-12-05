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

    /// Tests footer displays all required keyboard shortcuts.
    #[test]
    fn test_footer_displays_all_shortcuts() {
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

            // All expected shortcuts must be present
            let expected_shortcuts = [
                "q:Quit",
                "r:Refresh",
                "f:Search",
                "n:Network",
                "Space:Live",
                "Tab:Focus",
            ];

            for shortcut in expected_shortcuts {
                assert!(
                    content.contains(shortcut),
                    "Footer should contain '{}', got: {}",
                    shortcut,
                    content
                );
            }
        });
    }

    /// Tests footer rendering handles edge cases and maintains centering.
    #[test]
    fn test_footer_rendering_robustness() {
        test_with_mock_app(|app| {
            // Test narrow width doesn't panic
            {
                let backend = TestBackend::new(40, 1);
                let mut terminal = Terminal::new(backend).unwrap();
                terminal
                    .draw(|frame| render(frame, frame.area(), app))
                    .unwrap();
                assert_eq!(terminal.backend().buffer().area().width, 40);
            }

            // Test zero height doesn't panic
            {
                let backend = TestBackend::new(80, 1);
                let mut terminal = Terminal::new(backend).unwrap();
                terminal
                    .draw(|frame| render(frame, Rect::new(0, 0, 80, 0), app))
                    .unwrap();
            }

            // Test centering at normal width
            {
                let backend = TestBackend::new(80, 1);
                let mut terminal = Terminal::new(backend).unwrap();
                terminal
                    .draw(|frame| render(frame, frame.area(), app))
                    .unwrap();

                let buffer = terminal.backend().buffer();
                let content = buffer_to_string(buffer, 80, 1);
                let trimmed = content.trim();

                let leading_spaces = content.len() - content.trim_start().len();
                let trailing_spaces = content.len() - content.trim_end().len();
                let diff = (leading_spaces as i32 - trailing_spaces as i32).abs();

                assert!(
                    diff <= 1,
                    "Text should be centered. Leading: {}, Trailing: {}",
                    leading_spaces,
                    trailing_spaces
                );
                assert!(!trimmed.is_empty(), "Footer should have content");
            }
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
