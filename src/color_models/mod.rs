pub mod eight_bit;
pub mod four_bit;
pub mod twenty_four_bit;

// Re-export the main functions for easier external use
pub use eight_bit::parse_8bit_color;
pub use four_bit::parse_4bit_color;
pub use twenty_four_bit::parse_24bit_color;
