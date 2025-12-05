//! Command handler trait and implementations for state management.
//!
//! This module provides the [`CommandHandler`] trait for executing application
//! commands in a structured way, along with [`CommandResult`] for representing
//! the outcome of command execution.
//!
//! # Architecture
//!
//! The command handler pattern separates command execution from command dispatch:
//!
//! ```text
//! KeyEvent -> KeyMapper -> AppCommand -> CommandHandler -> StateChange
//! ```
//!
//! This allows for:
//! - Testable command execution logic
//! - Potential undo/redo support
//! - Command logging and analytics
//! - Async command execution
//!
//! # Example
//!
//! ```ignore
//! use crate::state::{CommandHandler, CommandContext, CommandResult};
//!
//! struct MyHandler;
//!
//! impl CommandHandler for MyHandler {
//!     fn handle(&mut self, ctx: &mut CommandContext) -> CommandResult {
//!         // Execute command logic
//!         CommandResult::success()
//!     }
//! }
//! ```

// TODO: Remove after full integration in Stage 2
#![allow(dead_code)]

use std::fmt;

use crate::commands::AppCommand;
use crate::state::{DataState, NavigationState, UiState};

// ============================================================================
// Command Result
// ============================================================================

/// The result of executing a command.
///
/// Commands can succeed, fail, or request specific follow-up actions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandResult {
    /// Command executed successfully with no additional action needed.
    Success,
    /// Command executed successfully and requests a UI refresh.
    Refresh,
    /// Command executed successfully and requests application exit.
    Exit,
    /// Command failed with an error message.
    Error(String),
    /// Command was not handled (should be passed to another handler).
    NotHandled,
    /// Command requires async work (returns a task identifier).
    Async(String),
}

impl CommandResult {
    /// Creates a successful result.
    #[must_use]
    pub const fn success() -> Self {
        Self::Success
    }

    /// Creates a result that requests a UI refresh.
    #[must_use]
    pub const fn refresh() -> Self {
        Self::Refresh
    }

    /// Creates a result that requests application exit.
    #[must_use]
    pub const fn exit() -> Self {
        Self::Exit
    }

    /// Creates an error result with a message.
    #[must_use]
    pub fn error(message: impl Into<String>) -> Self {
        Self::Error(message.into())
    }

    /// Creates a not-handled result.
    #[must_use]
    pub const fn not_handled() -> Self {
        Self::NotHandled
    }

    /// Creates an async result with a task identifier.
    #[must_use]
    pub fn async_task(task_id: impl Into<String>) -> Self {
        Self::Async(task_id.into())
    }

    /// Returns `true` if the command was handled (success or error).
    #[must_use]
    pub const fn was_handled(&self) -> bool {
        !matches!(self, Self::NotHandled)
    }

    /// Returns `true` if the command succeeded.
    #[must_use]
    pub const fn is_success(&self) -> bool {
        matches!(
            self,
            Self::Success | Self::Refresh | Self::Exit | Self::Async(_)
        )
    }

    /// Returns `true` if the command failed.
    #[must_use]
    pub const fn is_error(&self) -> bool {
        matches!(self, Self::Error(_))
    }

    /// Returns `true` if the command requests exit.
    #[must_use]
    pub const fn should_exit(&self) -> bool {
        matches!(self, Self::Exit)
    }

    /// Returns `true` if the command requests a refresh.
    #[must_use]
    pub const fn should_refresh(&self) -> bool {
        matches!(self, Self::Refresh)
    }

    /// Returns the error message if this is an error result.
    #[must_use]
    pub fn error_message(&self) -> Option<&str> {
        match self {
            Self::Error(msg) => Some(msg.as_str()),
            _ => None,
        }
    }

    /// Returns the async task ID if this is an async result.
    #[must_use]
    pub fn async_task_id(&self) -> Option<&str> {
        match self {
            Self::Async(id) => Some(id.as_str()),
            _ => None,
        }
    }
}

impl fmt::Display for CommandResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Success => write!(f, "Success"),
            Self::Refresh => write!(f, "Refresh requested"),
            Self::Exit => write!(f, "Exit requested"),
            Self::Error(msg) => write!(f, "Error: {msg}"),
            Self::NotHandled => write!(f, "Not handled"),
            Self::Async(id) => write!(f, "Async task: {id}"),
        }
    }
}

// ============================================================================
// Command Context
// ============================================================================

/// Context provided to command handlers for accessing and modifying state.
///
/// The context provides mutable access to the decomposed state components,
/// allowing handlers to make targeted state changes.
#[derive(Debug)]
pub struct CommandContext<'a> {
    /// Navigation state (selections, scroll positions, detail views).
    pub nav: &'a mut NavigationState,
    /// Data state (blocks, transactions, search results).
    pub data: &'a mut DataState,
    /// UI state (focus, popups, viewing flags).
    pub ui: &'a mut UiState,
    /// The command being executed.
    pub command: AppCommand,
}

impl<'a> CommandContext<'a> {
    /// Creates a new command context.
    ///
    /// # Arguments
    ///
    /// * `nav` - Mutable reference to navigation state
    /// * `data` - Mutable reference to data state
    /// * `ui` - Mutable reference to UI state
    /// * `command` - The command to execute
    #[must_use]
    pub fn new(
        nav: &'a mut NavigationState,
        data: &'a mut DataState,
        ui: &'a mut UiState,
        command: AppCommand,
    ) -> Self {
        Self {
            nav,
            data,
            ui,
            command,
        }
    }

    /// Returns `true` if any detail view is showing.
    #[must_use]
    pub fn is_showing_details(&self) -> bool {
        self.nav.is_showing_details()
    }

    /// Returns `true` if a popup is active.
    #[must_use]
    pub fn has_active_popup(&self) -> bool {
        self.ui.has_active_popup()
    }
}

// ============================================================================
// Command Handler Trait
// ============================================================================

/// Trait for handling application commands.
///
/// Implementations of this trait can execute commands and modify application
/// state through the provided context.
///
/// # Example
///
/// ```ignore
/// struct QuitHandler;
///
/// impl CommandHandler for QuitHandler {
///     fn can_handle(&self, command: &AppCommand) -> bool {
///         matches!(command, AppCommand::Quit)
///     }
///
///     fn handle(&mut self, ctx: &mut CommandContext) -> CommandResult {
///         CommandResult::exit()
///     }
/// }
/// ```
pub trait CommandHandler {
    /// Returns `true` if this handler can handle the given command.
    ///
    /// The default implementation returns `true` for all commands.
    fn can_handle(&self, _command: &AppCommand) -> bool {
        true
    }

    /// Handles the command and returns the result.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The command context containing state references
    ///
    /// # Returns
    ///
    /// The result of executing the command.
    fn handle(&mut self, ctx: &mut CommandContext) -> CommandResult;

    /// Returns a description of this handler for debugging.
    fn description(&self) -> &'static str {
        "CommandHandler"
    }
}

// ============================================================================
// Composite Command Handler
// ============================================================================

/// A handler that chains multiple handlers together.
///
/// Commands are passed to each handler in order until one handles it.
/// If no handler handles the command, `NotHandled` is returned.
pub struct CompositeHandler {
    handlers: Vec<Box<dyn CommandHandler>>,
}

impl CompositeHandler {
    /// Creates a new composite handler with no child handlers.
    #[must_use]
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }

    /// Adds a handler to the chain.
    pub fn add_handler(&mut self, handler: impl CommandHandler + 'static) {
        self.handlers.push(Box::new(handler));
    }

    /// Creates a composite handler with the given handlers.
    #[must_use]
    pub fn with_handlers(handlers: Vec<Box<dyn CommandHandler>>) -> Self {
        Self { handlers }
    }
}

impl Default for CompositeHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandHandler for CompositeHandler {
    fn can_handle(&self, command: &AppCommand) -> bool {
        self.handlers.iter().any(|h| h.can_handle(command))
    }

    fn handle(&mut self, ctx: &mut CommandContext) -> CommandResult {
        for handler in &mut self.handlers {
            if handler.can_handle(&ctx.command) {
                let result = handler.handle(ctx);
                if result.was_handled() {
                    return result;
                }
            }
        }
        CommandResult::not_handled()
    }

    fn description(&self) -> &'static str {
        "CompositeHandler"
    }
}

// ============================================================================
// No-Op Handler
// ============================================================================

/// A handler that does nothing (for testing or placeholder purposes).
#[derive(Debug, Default, Clone, Copy)]
pub struct NoOpHandler;

impl CommandHandler for NoOpHandler {
    fn can_handle(&self, command: &AppCommand) -> bool {
        matches!(command, AppCommand::Noop)
    }

    fn handle(&mut self, _ctx: &mut CommandContext) -> CommandResult {
        CommandResult::success()
    }

    fn description(&self) -> &'static str {
        "NoOpHandler"
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    mod command_result_tests {
        use super::*;

        #[test]
        fn test_success_is_success() {
            let result = CommandResult::success();
            assert!(result.is_success());
            assert!(!result.is_error());
            assert!(result.was_handled());
        }

        #[test]
        fn test_refresh_is_success() {
            let result = CommandResult::refresh();
            assert!(result.is_success());
            assert!(result.should_refresh());
            assert!(result.was_handled());
        }

        #[test]
        fn test_exit_requests_exit() {
            let result = CommandResult::exit();
            assert!(result.should_exit());
            assert!(result.is_success());
            assert!(result.was_handled());
        }

        #[test]
        fn test_error_is_error() {
            let result = CommandResult::error("test error");
            assert!(result.is_error());
            assert!(!result.is_success());
            assert!(result.was_handled());
            assert_eq!(result.error_message(), Some("test error"));
        }

        #[test]
        fn test_not_handled_is_not_handled() {
            let result = CommandResult::not_handled();
            assert!(!result.was_handled());
            assert!(!result.is_success());
            assert!(!result.is_error());
        }

        #[test]
        fn test_async_has_task_id() {
            let result = CommandResult::async_task("task_123");
            assert!(result.is_success());
            assert_eq!(result.async_task_id(), Some("task_123"));
        }

        #[test]
        fn test_display_formatting() {
            assert_eq!(CommandResult::success().to_string(), "Success");
            assert_eq!(CommandResult::refresh().to_string(), "Refresh requested");
            assert_eq!(CommandResult::exit().to_string(), "Exit requested");
            assert_eq!(CommandResult::error("oops").to_string(), "Error: oops");
            assert_eq!(CommandResult::not_handled().to_string(), "Not handled");
            assert_eq!(
                CommandResult::async_task("task_1").to_string(),
                "Async task: task_1"
            );
        }
    }

    mod noop_handler_tests {
        use super::*;

        #[test]
        fn test_handles_noop_command() {
            assert!(NoOpHandler.can_handle(&AppCommand::Noop));
        }

        #[test]
        fn test_does_not_handle_other_commands() {
            assert!(!NoOpHandler.can_handle(&AppCommand::Quit));
            assert!(!NoOpHandler.can_handle(&AppCommand::Refresh));
            assert!(!NoOpHandler.can_handle(&AppCommand::MoveUp));
        }

        #[test]
        fn test_returns_success() {
            let mut handler = NoOpHandler;
            let mut nav = NavigationState::new();
            let mut data = DataState::new();
            let mut ui = UiState::new();
            let mut ctx = CommandContext::new(&mut nav, &mut data, &mut ui, AppCommand::Noop);

            let result = handler.handle(&mut ctx);
            assert!(result.is_success());
        }

        #[test]
        fn test_description() {
            assert_eq!(NoOpHandler.description(), "NoOpHandler");
        }
    }

    mod composite_handler_tests {
        use super::*;

        struct AlwaysSuccessHandler;

        impl CommandHandler for AlwaysSuccessHandler {
            fn handle(&mut self, _ctx: &mut CommandContext) -> CommandResult {
                CommandResult::success()
            }

            fn description(&self) -> &'static str {
                "AlwaysSuccessHandler"
            }
        }

        struct AlwaysErrorHandler;

        impl CommandHandler for AlwaysErrorHandler {
            fn handle(&mut self, _ctx: &mut CommandContext) -> CommandResult {
                CommandResult::error("always fails")
            }

            fn description(&self) -> &'static str {
                "AlwaysErrorHandler"
            }
        }

        struct SelectiveHandler {
            target: AppCommand,
        }

        impl CommandHandler for SelectiveHandler {
            fn can_handle(&self, command: &AppCommand) -> bool {
                *command == self.target
            }

            fn handle(&mut self, _ctx: &mut CommandContext) -> CommandResult {
                CommandResult::success()
            }

            fn description(&self) -> &'static str {
                "SelectiveHandler"
            }
        }

        #[test]
        fn test_empty_composite_returns_not_handled() {
            let mut handler = CompositeHandler::new();
            let mut nav = NavigationState::new();
            let mut data = DataState::new();
            let mut ui = UiState::new();
            let mut ctx = CommandContext::new(&mut nav, &mut data, &mut ui, AppCommand::Quit);

            let result = handler.handle(&mut ctx);
            assert!(!result.was_handled());
        }

        #[test]
        fn test_first_handler_success_stops_chain() {
            let mut handler = CompositeHandler::new();
            handler.add_handler(AlwaysSuccessHandler);
            handler.add_handler(AlwaysErrorHandler);

            let mut nav = NavigationState::new();
            let mut data = DataState::new();
            let mut ui = UiState::new();
            let mut ctx = CommandContext::new(&mut nav, &mut data, &mut ui, AppCommand::Quit);

            let result = handler.handle(&mut ctx);
            assert!(result.is_success());
        }

        #[test]
        fn test_selective_handler_routing() {
            let mut handler = CompositeHandler::new();
            handler.add_handler(SelectiveHandler {
                target: AppCommand::MoveUp,
            });
            handler.add_handler(SelectiveHandler {
                target: AppCommand::MoveDown,
            });

            // Test that composite can handle both commands
            assert!(handler.can_handle(&AppCommand::MoveUp));
            assert!(handler.can_handle(&AppCommand::MoveDown));
            assert!(!handler.can_handle(&AppCommand::Quit));
        }

        #[test]
        fn test_default_creates_empty() {
            let handler = CompositeHandler::default();
            assert!(handler.handlers.is_empty());
        }
    }

    mod command_context_tests {
        use super::*;

        #[test]
        fn test_context_creation() {
            let mut nav = NavigationState::new();
            let mut data = DataState::new();
            let mut ui = UiState::new();
            let ctx = CommandContext::new(&mut nav, &mut data, &mut ui, AppCommand::Quit);

            assert_eq!(ctx.command, AppCommand::Quit);
        }

        #[test]
        fn test_is_showing_details_delegates() {
            let mut nav = NavigationState::new();
            let mut data = DataState::new();
            let mut ui = UiState::new();

            let ctx = CommandContext::new(&mut nav, &mut data, &mut ui, AppCommand::Quit);
            assert!(!ctx.is_showing_details());
        }

        #[test]
        fn test_has_active_popup_delegates() {
            let mut nav = NavigationState::new();
            let mut data = DataState::new();
            let mut ui = UiState::new();

            let ctx = CommandContext::new(&mut nav, &mut data, &mut ui, AppCommand::Quit);
            assert!(!ctx.has_active_popup());
        }
    }
}
