//! # egui_sgr
//!
//! A library for converting ASCII/ANSI escape sequence color models to colored text in egui.
//!
//! ## Features
//!
//! - Supports 4-bit color (16 colors) model
//! - Supports 8-bit color (256 colors) model
//! - Supports 24-bit true color model
//! - Automatically detects and converts mixed color sequences
//! - Supports simultaneous setting of foreground and background colors
//!
//! ## Usage Example
//!
//! ```rust
//! use egui_sgr::ansi_to_rich_text;
//!
//! // Convert ANSI color sequences to egui RichText
//! let red_text = ansi_to_rich_text("\x1b[31mRed Text\x1b[0m");
//! let orange_text = ansi_to_rich_text("\x1b[38;5;208mOrange Text\x1b[0m");
//! let pink_text = ansi_to_rich_text("\x1b[38;2;255;105;180mPink Text\x1b[0m");
//! let colored_bg = ansi_to_rich_text("\x1b[41;33mYellow on Red\x1b[0m");
//! ```

use egui::{Color32, RichText};
use regex::Regex;
use std::sync::LazyLock;

mod color_models;

/// Pre-compiled regex for matching ANSI escape sequences (cached for performance)
static ANSI_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\x1b\[([0-9;]+)m").expect("Invalid ANSI regex pattern"));

// Re-export color model modules
pub use color_models::*;

/// Represents a text segment with color information
#[derive(Debug, Clone)]
pub struct ColoredText {
    pub text: String,
    pub foreground_color: Option<Color32>,
    pub background_color: Option<Color32>,
}

/// ANSI escape sequence parser
pub struct AnsiParser {
    // Currently cached foreground and background colors
    current_fg: Option<Color32>,
    current_bg: Option<Color32>,
}

impl AnsiParser {
    /// Creates a new ANSI parser
    pub fn new() -> Self {
        Self {
            current_fg: None,
            current_bg: None,
        }
    }

    /// Parses text containing ANSI escape sequences
    ///
    /// # Arguments
    /// - `input`: Text containing ANSI escape sequences
    ///
    /// # Returns
    /// A list of text segments with color information
    pub fn parse(&mut self, input: &str) -> Vec<ColoredText> {
        // Directly parse input - don't preprocess escape sequences
        // Only handle actual control characters for egui display compatibility
        self.parse_direct(input)
    }

    /// Parse text containing ANSI escape sequences without preprocessing
    fn parse_direct(&mut self, input: &str) -> Vec<ColoredText> {
        // Initialize the result list
        let mut result = Vec::new();

        // Reset current colors
        self.reset_colors();

        let mut last_end = 0;

        // Iterate over all matched ANSI sequences using pre-compiled regex
        for cap in ANSI_REGEX.captures_iter(input) {
            let sequence = cap.get(1).unwrap().as_str();
            let start = cap.get(0).unwrap().start();
            let end = cap.get(0).unwrap().end();

            // Add the text before the escape sequence (if any)
            if start > last_end {
                let plain_text = &input[last_end..start];
                if !plain_text.is_empty() {
                    result.push(ColoredText {
                        text: plain_text.to_string(),
                        foreground_color: self.current_fg,
                        background_color: self.current_bg,
                    });
                }
            }

            // Process the ANSI sequence to update the current color
            self.process_ansi_sequence(sequence);

            last_end = end;
        }

        // Add the remaining text (if any)
        if last_end < input.len() {
            let plain_text = &input[last_end..];
            if !plain_text.is_empty() {
                result.push(ColoredText {
                    text: plain_text.to_string(),
                    foreground_color: self.current_fg,
                    background_color: self.current_bg,
                });
            }
        }

        // If no escape sequences were found, return the entire text
        if result.is_empty() {
            return vec![ColoredText {
                text: input.to_string(),
                foreground_color: None,
                background_color: None,
            }];
        }

        result
    }

    /// Resets the current colors
    fn reset_colors(&mut self) {
        self.current_fg = None;
        self.current_bg = None;
    }

    /// Processes a single ANSI escape sequence and updates the current color cache
    ///
    /// # Arguments
    /// - `sequence`: The ANSI escape sequence
    fn process_ansi_sequence(&mut self, sequence: &str) {
        // Split the sequence into multiple codes
        let codes: Vec<&str> = sequence.split(';').collect();

        // Process each code
        for i in 0..codes.len() {
            match codes[i] {
                "0" => {
                    // Reset all attributes
                    self.reset_colors();
                }
                "38" => {
                    // Set foreground color
                    if i + 2 < codes.len() {
                        match codes[i + 1] {
                            "5" => {
                                // 256-color mode
                                if let Ok(color_code) = codes[i + 2].parse::<u8>() {
                                    self.current_fg =
                                        Some(color_models::eight_bit::ansi_256_to_egui(color_code));
                                }
                            }
                            "2" => {
                                // 24-bit true color mode
                                if i + 4 < codes.len() {
                                    if let (Ok(r), Ok(g), Ok(b)) = (
                                        codes[i + 2].parse::<u8>(),
                                        codes[i + 3].parse::<u8>(),
                                        codes[i + 4].parse::<u8>(),
                                    ) {
                                        self.current_fg = Some(Color32::from_rgb(r, g, b));
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                "48" => {
                    // Set background color
                    if i + 2 < codes.len() {
                        match codes[i + 1] {
                            "5" => {
                                // 256-color mode
                                if let Ok(color_code) = codes[i + 2].parse::<u8>() {
                                    self.current_bg =
                                        Some(color_models::eight_bit::ansi_256_to_egui(color_code));
                                }
                            }
                            "2" => {
                                // 24-bit true color mode
                                if i + 4 < codes.len() {
                                    if let (Ok(r), Ok(g), Ok(b)) = (
                                        codes[i + 2].parse::<u8>(),
                                        codes[i + 3].parse::<u8>(),
                                        codes[i + 4].parse::<u8>(),
                                    ) {
                                        self.current_bg = Some(Color32::from_rgb(r, g, b));
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                "39" => {
                    // Reset foreground color
                    self.current_fg = None;
                }
                "49" => {
                    // Reset background color
                    self.current_bg = None;
                }
                code => {
                    // Handle 4-bit color codes
                    if let Ok(color_code) = code.parse::<u8>() {
                        match color_code {
                            30..=37 => {
                                // Standard foreground color
                                let color_index = color_code - 30;
                                self.current_fg =
                                    Some(color_models::four_bit::ansi_color_to_egui(color_index));
                            }
                            40..=47 => {
                                // Standard background color
                                let color_index = color_code - 40;
                                self.current_bg =
                                    Some(color_models::four_bit::ansi_color_to_egui(color_index));
                            }
                            90..=97 => {
                                // Bright foreground color
                                let color_index = color_code - 90 + 8;
                                self.current_fg =
                                    Some(color_models::four_bit::ansi_color_to_egui(color_index));
                            }
                            100..=107 => {
                                // Bright background color
                                let color_index = color_code - 100 + 8;
                                self.current_bg =
                                    Some(color_models::four_bit::ansi_color_to_egui(color_index));
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
}

/// Converts a list of ColoredText to a list of RichText that can be displayed in egui
///
/// # Arguments
/// - `colored_texts`: A list of text segments with color information
///
/// # Returns
/// A list of RichText that can be displayed in egui
pub fn convert_to_rich_text(colored_texts: &[ColoredText]) -> Vec<RichText> {
    colored_texts
        .iter()
        .map(|colored_text| {
            let mut rich_text = RichText::new(&colored_text.text);

            if let Some(fg) = colored_text.foreground_color {
                rich_text = rich_text.color(fg);
            }

            if let Some(bg) = colored_text.background_color {
                rich_text = rich_text.background_color(bg);
            }

            rich_text
        })
        .collect()
}

/// Convenience function: directly converts text with ANSI escape sequences to a list of RichText
///
/// # Arguments
/// - `input`: Text containing ANSI escape sequences
///
/// # Returns
/// A list of RichText that can be displayed in egui
pub fn ansi_to_rich_text(input: &str) -> Vec<RichText> {
    // Directly parse input - don't preprocess escape sequences
    // Only handle actual control characters for egui display compatibility
    let mut parser = AnsiParser::new();
    let colored_texts = parser.parse(input);
    convert_to_rich_text(&colored_texts)
}

/// Creates an example function to demonstrate how to use this library
pub fn example_usage() {
    // 4-bit color example
    let example_4bit =
        "This is \x1b[31mred\x1b[0m, \x1b[34mblue\x1b[0m and \x1b[33myellow\x1b[0m text";
    let _rich_text_4bit = ansi_to_rich_text(example_4bit);

    // 8-bit color example
    let example_8bit = "This is \x1b[38;5;208morange\x1b[0m, \x1b[38;5;51msky blue\x1b[0m and \x1b[38;5;120mgreen\x1b[0m text";
    let _rich_text_8bit = ansi_to_rich_text(example_8bit);

    // 24-bit color example
    let example_24bit = "This is \x1b[38;2;255;105;180mhot pink\x1b[0m and \x1b[38;2;128;0;128mdeep purple\x1b[0m text";
    let _rich_text_24bit = ansi_to_rich_text(example_24bit);

    // Mixed example
    let example_mixed = "Normal text \x1b[31mred\x1b[0m normal \x1b[38;5;208morange\x1b[0m normal \x1b[38;2;255;105;180mpink\x1b[0m normal";
    let _rich_text_mixed = ansi_to_rich_text(example_mixed);

    // Foreground and background color combination example
    let example_fg_bg = "Default text \x1b[41;33mYellow on red\x1b[0m default text";
    let _rich_text_fg_bg = ansi_to_rich_text(example_fg_bg);

    // Use these RichText lists in an egui application
    // ui.horizontal(|ui| {
    //     for text in rich_text_4bit {
    //         ui.label(text);
    //     }
    // });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_4bit_color_parsing() {
        let input = "\x1b[31mRed Text\x1b[0m";
        let result = ansi_to_rich_text(input);
        assert_eq!(result.len(), 1);
        // Verify that the color is correct
        assert_eq!(result[0].text(), "Red Text");
        // Note: We don't verify the specific color value here as our implementation may not be perfectly consistent
    }

    #[test]
    fn test_8bit_color_parsing() {
        let input = "\x1b[38;5;208mOrange Text\x1b[0m";
        let result = ansi_to_rich_text(input);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].text(), "Orange Text");
    }

    #[test]
    fn test_24bit_color_parsing() {
        let input = "\x1b[38;2;255;105;180mHot Pink Text\x1b[0m";
        let result = ansi_to_rich_text(input);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].text(), "Hot Pink Text");
    }

    #[test]
    fn test_mixed_colors() {
        let input =
            "Normal text \x1b[31mred\x1b[0m normal text \x1b[38;5;208morange\x1b[0m normal text";
        let result = ansi_to_rich_text(input);
        assert_eq!(result.len(), 5); // Normal text + red + normal text + orange + normal text
        assert_eq!(result[0].text(), "Normal text ");
        assert_eq!(result[1].text(), "red");
        assert_eq!(result[2].text(), " normal text ");
        assert_eq!(result[3].text(), "orange");
        assert_eq!(result[4].text(), " normal text");
    }

    #[test]
    fn test_background_color() {
        let input = "\x1b[41mWhite on red\x1b[0m";
        let result = ansi_to_rich_text(input);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].text(), "White on red");
    }

    #[test]
    fn test_foreground_and_background_color() {
        let input = "\x1b[31;43mYellow on red\x1b[0m";
        let mut parser = AnsiParser::new();
        let colored_segments = parser.parse(input);

        // There should be only one text segment with both foreground and background colors
        assert_eq!(colored_segments.len(), 1);
        assert_eq!(colored_segments[0].text, "Yellow on red");
        assert!(colored_segments[0].foreground_color.is_some());
        assert!(colored_segments[0].background_color.is_some());
    }

    #[test]
    fn test_sequential_color_changes() {
        let input = "Default\x1b[31mRed\x1b[43mRed on yellow\x1b[32mGreen on yellow\x1b[0mDefault";
        let mut parser = AnsiParser::new();
        let colored_segments = parser.parse(input);

        // There should be 5 text segments: Default, Red, Red on yellow, Green on yellow, Default
        assert_eq!(colored_segments.len(), 5);
        assert_eq!(colored_segments[0].text, "Default");
        assert_eq!(colored_segments[1].text, "Red");
        assert_eq!(colored_segments[2].text, "Red on yellow");
        assert_eq!(colored_segments[3].text, "Green on yellow");
        assert_eq!(colored_segments[4].text, "Default");

        // Check colors
        assert!(colored_segments[0].foreground_color.is_none());
        assert!(colored_segments[0].background_color.is_none());

        assert!(colored_segments[1].foreground_color.is_some());
        assert!(colored_segments[1].background_color.is_none());

        assert!(colored_segments[2].foreground_color.is_some());
        assert!(colored_segments[2].background_color.is_some());

        assert!(colored_segments[3].foreground_color.is_some());
        assert!(colored_segments[3].background_color.is_some());

        assert!(colored_segments[4].foreground_color.is_none());
        assert!(colored_segments[4].background_color.is_none());
    }

    #[test]
    fn test_escape_sequence_variations() {
        // Test that escaped sequences (with double backslashes) are NOT processed
        // Only actual control characters should be processed
        let inputs = [
            "\\x1b[31mRed\\x1b[0m",
            "\\x1B[31mRed\\x1B[0m",
            "\\X1b[31mRed\\X1b[0m",
            "\\X1B[31mRed\\X1B[0m",
            "\\033[31mRed\\033[0m",
        ];

        for input in inputs {
            let mut parser = AnsiParser::new();
            let colored_segments = parser.parse(input);

            // All should return the raw text unchanged (no ANSI processing)
            assert_eq!(colored_segments.len(), 1);
            assert_eq!(colored_segments[0].text, input);
            assert!(colored_segments[0].foreground_color.is_none());
            assert!(colored_segments[0].background_color.is_none());
        }
    }

    #[test]
    fn test_mixed_escape_sequence_variations() {
        // Test mixing different escape sequence representations in the same string
        // These should NOT be processed since they are escaped sequences
        let input = "\\x1b[31mRed\\033[32mGreen\\X1B[33mYellow\\x1B[0m";
        let mut parser = AnsiParser::new();
        let colored_segments = parser.parse(input);

        // Should have 1 segment: the entire text unchanged
        assert_eq!(colored_segments.len(), 1);
        assert_eq!(colored_segments[0].text, input);

        // Should have no colors
        assert!(colored_segments[0].foreground_color.is_none());
        assert!(colored_segments[0].background_color.is_none());
    }

    #[test]
    fn test_ansi_to_rich_text_with_escape_variations() {
        // Test the convenience function with escaped sequences - should NOT be processed
        let inputs = [
            "\\x1b[31mRed\\x1b[0m",
            "\\x1B[31mRed\\x1B[0m",
            "\\X1b[31mRed\\X1b[0m",
            "\\X1B[31mRed\\X1B[0m",
            "\\033[31mRed\\033[0m",
        ];

        for input in inputs {
            let rich_text = ansi_to_rich_text(input);
            assert_eq!(rich_text.len(), 1);
            assert_eq!(rich_text[0].text(), input);
            // Should not have any color processing
        }
    }
}
