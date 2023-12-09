//! ANSI terminal codes
//!
//! For more information, see <https://en.wikipedia.org/wiki/ANSI_escape_code>

/// ANSI code to clear line
pub const CLEAR_LINE: &str = "\x1b[2K";

/// ANSI code to clear screen
pub const CLEAR_SCREEN: &str = "\x1b[H\x1b[2J\x1b[3J";
