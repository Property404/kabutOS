//! ANSI terminal codes
//!
//! For more information, see <https://en.wikipedia.org/wiki/ANSI_escape_code>

/// ANSI code to clear line
pub const CLEAR_LINE: &str = "\x1b[2K";

/// ANSI color codes
pub mod colors {
    /// Reset color
    pub const RESET: &str = "\x1b[0m";
    /// Red
    pub const RED: &str = "\x1b[31m";
    /// Blue
    pub const BLUE: &str = "\x1b[32m";
    /// Green
    pub const GREEN: &str = "\x1b[33m";
    /// Purple
    pub const PURPLE: &str = "\x1b[35m";
}
