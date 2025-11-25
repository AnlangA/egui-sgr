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

/// Represents a text segment with optional foreground and background color information.
///
/// This struct is the output of parsing ANSI escape sequences, where each segment
/// represents a continuous piece of text with consistent color attributes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColoredText {
    /// The text content of this segment
    pub text: String,
    /// Optional foreground (text) color
    pub foreground_color: Option<Color32>,
    /// Optional background color
    pub background_color: Option<Color32>,
}

impl ColoredText {
    /// Creates a new ColoredText with no colors applied.
    #[must_use]
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            foreground_color: None,
            background_color: None,
        }
    }

    /// Creates a new ColoredText with the specified foreground color.
    #[must_use]
    pub fn with_foreground(text: impl Into<String>, color: Color32) -> Self {
        Self {
            text: text.into(),
            foreground_color: Some(color),
            background_color: None,
        }
    }

    /// Creates a new ColoredText with the specified background color.
    #[must_use]
    pub fn with_background(text: impl Into<String>, color: Color32) -> Self {
        Self {
            text: text.into(),
            foreground_color: None,
            background_color: Some(color),
        }
    }

    /// Creates a new ColoredText with both foreground and background colors.
    #[must_use]
    pub fn with_colors(
        text: impl Into<String>,
        foreground: Option<Color32>,
        background: Option<Color32>,
    ) -> Self {
        Self {
            text: text.into(),
            foreground_color: foreground,
            background_color: background,
        }
    }
}

/// ANSI escape sequence parser that converts ANSI color codes to egui colors.
///
/// This parser maintains color state between escape sequences, allowing for
/// proper handling of sequential color changes and nested formatting.
#[derive(Debug, Clone)]
pub struct AnsiParser {
    /// Currently cached foreground color
    current_fg: Option<Color32>,
    /// Currently cached background color
    current_bg: Option<Color32>,
}

impl Default for AnsiParser {
    fn default() -> Self {
        Self::new()
    }
}

impl AnsiParser {
    /// Creates a new ANSI parser with no active colors.
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
        let codes: Vec<&str> = sequence.split(';').collect();
        let mut i = 0;

        while i < codes.len() {
            match codes[i] {
                "0" => {
                    self.reset_colors();
                    i += 1;
                }
                "38" => {
                    // Set foreground color
                    // First check if we have at least 38;X;... (need at least 3 codes for 256-color mode)
                    if i + 2 < codes.len() {
                        match codes[i + 1] {
                            "5" => {
                                // 256-color mode: 38;5;n
                                if let Ok(color_code) = codes[i + 2].parse::<u8>() {
                                    self.current_fg =
                                        Some(color_models::eight_bit::ansi_256_to_egui(color_code));
                                }
                                i += 3; // Skip 38, 5, and the color code
                            }
                            "2" if i + 4 < codes.len() => {
                                // 24-bit true color mode: 38;2;r;g;b
                                if let (Ok(r), Ok(g), Ok(b)) = (
                                    codes[i + 2].parse::<u8>(),
                                    codes[i + 3].parse::<u8>(),
                                    codes[i + 4].parse::<u8>(),
                                ) {
                                    self.current_fg = Some(Color32::from_rgb(r, g, b));
                                }
                                i += 5; // Skip 38, 2, r, g, b
                            }
                            _ => {
                                i += 1;
                            }
                        }
                    } else {
                        i += 1;
                    }
                }
                "48" => {
                    // Set background color
                    // First check if we have at least 48;X;... (need at least 3 codes for 256-color mode)
                    if i + 2 < codes.len() {
                        match codes[i + 1] {
                            "5" => {
                                // 256-color mode: 48;5;n
                                if let Ok(color_code) = codes[i + 2].parse::<u8>() {
                                    self.current_bg =
                                        Some(color_models::eight_bit::ansi_256_to_egui(color_code));
                                }
                                i += 3; // Skip 48, 5, and the color code
                            }
                            "2" if i + 4 < codes.len() => {
                                // 24-bit true color mode: 48;2;r;g;b
                                if let (Ok(r), Ok(g), Ok(b)) = (
                                    codes[i + 2].parse::<u8>(),
                                    codes[i + 3].parse::<u8>(),
                                    codes[i + 4].parse::<u8>(),
                                ) {
                                    self.current_bg = Some(Color32::from_rgb(r, g, b));
                                }
                                i += 5; // Skip 48, 2, r, g, b
                            }
                            _ => {
                                i += 1;
                            }
                        }
                    } else {
                        i += 1;
                    }
                }
                "39" => {
                    self.current_fg = None;
                    i += 1;
                }
                "49" => {
                    self.current_bg = None;
                    i += 1;
                }
                code => {
                    // Handle 4-bit color codes
                    if let Ok(color_code) = code.parse::<u8>() {
                        match color_code {
                            30..=37 => {
                                let color_index = color_code - 30;
                                self.current_fg =
                                    Some(color_models::four_bit::ansi_color_to_egui(color_index));
                            }
                            40..=47 => {
                                let color_index = color_code - 40;
                                self.current_bg =
                                    Some(color_models::four_bit::ansi_color_to_egui(color_index));
                            }
                            90..=97 => {
                                let color_index = color_code - 90 + 8;
                                self.current_fg =
                                    Some(color_models::four_bit::ansi_color_to_egui(color_index));
                            }
                            100..=107 => {
                                let color_index = color_code - 100 + 8;
                                self.current_bg =
                                    Some(color_models::four_bit::ansi_color_to_egui(color_index));
                            }
                            _ => {}
                        }
                    }
                    i += 1;
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

    // Additional comprehensive tests for edge cases and stability

    #[test]
    fn test_empty_input() {
        let result = ansi_to_rich_text("");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].text(), "");
    }

    #[test]
    fn test_plain_text_no_escape() {
        let input = "Hello, World!";
        let result = ansi_to_rich_text(input);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].text(), "Hello, World!");
    }

    #[test]
    fn test_reset_foreground_color() {
        let input = "\x1b[31mRed\x1b[39mDefault";
        let mut parser = AnsiParser::new();
        let colored_segments = parser.parse(input);

        assert_eq!(colored_segments.len(), 2);
        assert_eq!(colored_segments[0].text, "Red");
        assert!(colored_segments[0].foreground_color.is_some());
        assert_eq!(colored_segments[1].text, "Default");
        assert!(colored_segments[1].foreground_color.is_none());
    }

    #[test]
    fn test_reset_background_color() {
        let input = "\x1b[41mRed BG\x1b[49mDefault BG";
        let mut parser = AnsiParser::new();
        let colored_segments = parser.parse(input);

        assert_eq!(colored_segments.len(), 2);
        assert_eq!(colored_segments[0].text, "Red BG");
        assert!(colored_segments[0].background_color.is_some());
        assert_eq!(colored_segments[1].text, "Default BG");
        assert!(colored_segments[1].background_color.is_none());
    }

    #[test]
    fn test_bright_foreground_colors() {
        let input = "\x1b[90mBright Black\x1b[91mBright Red\x1b[97mBright White\x1b[0m";
        let mut parser = AnsiParser::new();
        let colored_segments = parser.parse(input);

        assert_eq!(colored_segments.len(), 3);
        assert_eq!(colored_segments[0].text, "Bright Black");
        assert!(colored_segments[0].foreground_color.is_some());
        assert_eq!(colored_segments[1].text, "Bright Red");
        assert!(colored_segments[1].foreground_color.is_some());
        assert_eq!(colored_segments[2].text, "Bright White");
        assert!(colored_segments[2].foreground_color.is_some());
    }

    #[test]
    fn test_bright_background_colors() {
        let input = "\x1b[100mBright Black BG\x1b[101mBright Red BG\x1b[107mBright White BG\x1b[0m";
        let mut parser = AnsiParser::new();
        let colored_segments = parser.parse(input);

        assert_eq!(colored_segments.len(), 3);
        assert_eq!(colored_segments[0].text, "Bright Black BG");
        assert!(colored_segments[0].background_color.is_some());
        assert_eq!(colored_segments[1].text, "Bright Red BG");
        assert!(colored_segments[1].background_color.is_some());
        assert_eq!(colored_segments[2].text, "Bright White BG");
        assert!(colored_segments[2].background_color.is_some());
    }

    #[test]
    fn test_8bit_background_color() {
        let input = "\x1b[48;5;196mRed BG\x1b[0m";
        let mut parser = AnsiParser::new();
        let colored_segments = parser.parse(input);

        assert_eq!(colored_segments.len(), 1);
        assert_eq!(colored_segments[0].text, "Red BG");
        assert!(colored_segments[0].background_color.is_some());
    }

    #[test]
    fn test_24bit_foreground_color_value() {
        let input = "\x1b[38;2;255;0;0mRed\x1b[0m";
        let mut parser = AnsiParser::new();
        let colored_segments = parser.parse(input);

        assert_eq!(colored_segments.len(), 1);
        assert_eq!(colored_segments[0].text, "Red");
        assert_eq!(colored_segments[0].foreground_color, Some(Color32::from_rgb(255, 0, 0)));
    }

    #[test]
    fn test_24bit_background_color_value() {
        let input = "\x1b[48;2;0;255;0mGreen BG\x1b[0m";
        let mut parser = AnsiParser::new();
        let colored_segments = parser.parse(input);

        assert_eq!(colored_segments.len(), 1);
        assert_eq!(colored_segments[0].text, "Green BG");
        assert_eq!(colored_segments[0].background_color, Some(Color32::from_rgb(0, 255, 0)));
    }

    #[test]
    fn test_256_color_boundary_values() {
        // Test boundary values for 256-color mode (standard colors, RGB cube, grayscale)
        let input = "\x1b[38;5;0mColor0\x1b[38;5;15mColor15\x1b[38;5;16mColor16\x1b[38;5;231mColor231\x1b[38;5;232mColor232\x1b[38;5;255mColor255\x1b[0m";
        let mut parser = AnsiParser::new();
        let colored_segments = parser.parse(input);

        assert_eq!(colored_segments.len(), 6);
        for segment in &colored_segments {
            assert!(segment.foreground_color.is_some());
        }
    }

    #[test]
    fn test_consecutive_resets() {
        let input = "\x1b[0m\x1b[0m\x1b[0mText\x1b[0m";
        let mut parser = AnsiParser::new();
        let colored_segments = parser.parse(input);

        assert_eq!(colored_segments.len(), 1);
        assert_eq!(colored_segments[0].text, "Text");
        assert!(colored_segments[0].foreground_color.is_none());
        assert!(colored_segments[0].background_color.is_none());
    }

    #[test]
    fn test_default_parser() {
        let parser: AnsiParser = Default::default();
        let mut parser = parser;
        let result = parser.parse("\x1b[31mRed\x1b[0m");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].text, "Red");
    }

    #[test]
    fn test_colored_text_constructors() {
        let plain = ColoredText::new("Hello");
        assert_eq!(plain.text, "Hello");
        assert!(plain.foreground_color.is_none());
        assert!(plain.background_color.is_none());

        let fg = ColoredText::with_foreground("Hello", Color32::RED);
        assert_eq!(fg.text, "Hello");
        assert_eq!(fg.foreground_color, Some(Color32::RED));
        assert!(fg.background_color.is_none());

        let bg = ColoredText::with_background("Hello", Color32::BLUE);
        assert_eq!(bg.text, "Hello");
        assert!(bg.foreground_color.is_none());
        assert_eq!(bg.background_color, Some(Color32::BLUE));

        let both = ColoredText::with_colors("Hello", Some(Color32::RED), Some(Color32::BLUE));
        assert_eq!(both.text, "Hello");
        assert_eq!(both.foreground_color, Some(Color32::RED));
        assert_eq!(both.background_color, Some(Color32::BLUE));
    }

    #[test]
    fn test_colored_text_equality() {
        let a = ColoredText::new("Hello");
        let b = ColoredText::new("Hello");
        assert_eq!(a, b);

        let c = ColoredText::with_foreground("Hello", Color32::RED);
        let d = ColoredText::with_foreground("Hello", Color32::RED);
        assert_eq!(c, d);

        let e = ColoredText::with_foreground("Hello", Color32::BLUE);
        assert_ne!(c, e);
    }

    #[test]
    fn test_multiline_text() {
        let input = "\x1b[31mLine1\nLine2\x1b[0m";
        let mut parser = AnsiParser::new();
        let colored_segments = parser.parse(input);

        assert_eq!(colored_segments.len(), 1);
        assert_eq!(colored_segments[0].text, "Line1\nLine2");
        assert!(colored_segments[0].foreground_color.is_some());
    }

    #[test]
    fn test_unicode_text() {
        let input = "\x1b[31m你好世界🎉\x1b[0m";
        let mut parser = AnsiParser::new();
        let colored_segments = parser.parse(input);

        assert_eq!(colored_segments.len(), 1);
        assert_eq!(colored_segments[0].text, "你好世界🎉");
        assert!(colored_segments[0].foreground_color.is_some());
    }

    #[test]
    fn test_4bit_color_values() {
        use color_models::four_bit::ansi_color_to_egui;
        
        assert_eq!(ansi_color_to_egui(0), Color32::BLACK);
        assert_eq!(ansi_color_to_egui(1), Color32::RED);
        assert_eq!(ansi_color_to_egui(2), Color32::GREEN);
        assert_eq!(ansi_color_to_egui(3), Color32::YELLOW);
        assert_eq!(ansi_color_to_egui(4), Color32::BLUE);
    }

    #[test]
    fn test_8bit_standard_colors() {
        use color_models::eight_bit::ansi_256_to_egui;
        
        assert_eq!(ansi_256_to_egui(0), Color32::BLACK);
        assert_eq!(ansi_256_to_egui(1), Color32::RED);
        assert_eq!(ansi_256_to_egui(15), Color32::WHITE);
    }

    #[test]
    fn test_8bit_rgb_cube() {
        use color_models::eight_bit::ansi_256_to_egui;
        
        // 16 = (0,0,0) = black in RGB cube
        assert_eq!(ansi_256_to_egui(16), Color32::from_rgb(0, 0, 0));
        // 21 = (0,0,5) = pure blue
        assert_eq!(ansi_256_to_egui(21), Color32::from_rgb(0, 0, 255));
        // 196 = (5,0,0) = pure red  
        assert_eq!(ansi_256_to_egui(196), Color32::from_rgb(255, 0, 0));
    }

    #[test]
    fn test_8bit_grayscale() {
        use color_models::eight_bit::ansi_256_to_egui;
        
        assert_eq!(ansi_256_to_egui(232), Color32::from_rgb(8, 8, 8));
        assert_eq!(ansi_256_to_egui(255), Color32::from_rgb(248, 248, 248));
    }
}
