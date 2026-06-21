//! Legacy color model helpers used by the compatibility API.

/// 8-bit ANSI color helpers.
pub mod eight_bit;
/// 4-bit ANSI color helpers.
pub mod four_bit;
/// 24-bit ANSI true-color helpers.
pub mod twenty_four_bit;

// Re-export the main functions for easier external use
pub use eight_bit::parse_8bit_color;
pub use four_bit::parse_4bit_color;
pub use twenty_four_bit::parse_24bit_color;
