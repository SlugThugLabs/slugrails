//! ANSI styling and terminal detection helpers.

use std::io::IsTerminal;

/// Terminal styling helper with auto-detection for terminal color support.
#[derive(Debug, Clone, Copy)]
pub struct Style {
    enabled: bool,
}

impl Style {
    /// Detects whether stdout is a terminal and initializes styling accordingly.
    #[must_use]
    pub fn stdout() -> Self {
        Self {
            enabled: std::io::stdout().is_terminal(),
        }
    }

    /// Creates a plain style instance with terminal colors unconditionally disabled.
    #[must_use]
    pub fn plain() -> Self {
        Self { enabled: false }
    }

    /// Checks if styling is currently enabled.
    #[must_use]
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    /// Formats the given text in bold if styling is enabled.
    #[must_use]
    pub fn bold(&self, text: &str) -> String {
        if self.enabled {
            format!("\x1b[1m{text}\x1b[0m")
        } else {
            text.to_string()
        }
    }

    /// Formats the given text in dim/faint style if styling is enabled.
    #[must_use]
    pub fn dim(&self, text: &str) -> String {
        if self.enabled {
            format!("\x1b[2m{text}\x1b[0m")
        } else {
            text.to_string()
        }
    }

    /// Formats the given text in cyan color if styling is enabled.
    #[must_use]
    pub fn cyan(&self, text: &str) -> String {
        if self.enabled {
            format!("\x1b[36m{text}\x1b[0m")
        } else {
            text.to_string()
        }
    }

    /// Formats the given text in green color if styling is enabled.
    #[must_use]
    pub fn green(&self, text: &str) -> String {
        if self.enabled {
            format!("\x1b[32m{text}\x1b[0m")
        } else {
            text.to_string()
        }
    }

    /// Formats the given text in yellow color if styling is enabled.
    #[must_use]
    pub fn yellow(&self, text: &str) -> String {
        if self.enabled {
            format!("\x1b[33m{text}\x1b[0m")
        } else {
            text.to_string()
        }
    }
}
