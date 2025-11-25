use egui::{Color32, RichText};

/// Standard 16-color palette for 4-bit ANSI color codes.
///
/// Colors 0-7 are standard colors (30-37 foreground, 40-47 background).
/// Colors 8-15 are bright variants (90-97 foreground, 100-107 background).
const COLORS: [Color32; 16] = [
    // 0-7: Standard colors
    Color32::BLACK,                   // Black (30/40)
    Color32::RED,                     // Red (31/41)
    Color32::GREEN,                   // Green (32/42)
    Color32::YELLOW,                  // Yellow (33/43)
    Color32::BLUE,                    // Blue (34/44)
    Color32::from_rgb(255, 0, 255),   // Magenta (35/45)
    Color32::from_rgb(0, 255, 255),   // Cyan (36/46)
    Color32::from_rgb(255, 255, 255), // White (37/47)
    // 8-15: Bright colors
    Color32::from_rgb(128, 128, 128), // Bright Black/Gray (90/100)
    Color32::from_rgb(255, 128, 128), // Bright Red (91/101)
    Color32::from_rgb(128, 255, 128), // Bright Green (92/102)
    Color32::from_rgb(255, 255, 128), // Bright Yellow (93/103)
    Color32::from_rgb(128, 128, 255), // Bright Blue (94/104)
    Color32::from_rgb(255, 128, 255), // Bright Magenta (95/105)
    Color32::from_rgb(128, 255, 255), // Bright Cyan (96/106)
    Color32::WHITE,                   // Bright White (97/107)
];

/// Converts an ANSI color code to an egui color
pub fn ansi_color_to_egui(color_code: u8) -> Color32 {
    if color_code < 16 {
        COLORS[color_code as usize]
    } else {
        // Default to black
        Color32::BLACK
    }
}

/// Applies a foreground color
pub fn apply_foreground_color(text: &str, color_code: u8) -> RichText {
    let color = ansi_color_to_egui(color_code);
    RichText::new(text).color(color)
}

/// Applies a background color
pub fn apply_background_color(text: &str, color_code: u8) -> RichText {
    let color = ansi_color_to_egui(color_code);
    RichText::new(text).background_color(color)
}

/// Parses a 4-bit color ANSI sequence and applies the color
///
/// # Arguments
/// - `text`: The text to render
/// - `sequence`: The ANSI color sequence, e.g., "31" (foreground red), "41" (background red)
/// - `is_background`: Whether it is a background color
///
/// # Returns
/// RichText with the color applied
pub fn parse_4bit_color(text: &str, sequence: &str, is_background: bool) -> Option<RichText> {
    // Matches standard 4-bit color sequences
    let re = regex::Regex::new(r"^([34][0-7]|9[0-7]|10[0-7])$").ok()?;

    if !re.is_match(sequence) {
        return None;
    }

    // Extract the color code
    let color_code = if let Ok(code) = sequence.parse::<u8>() {
        // Convert ANSI code to an index from 0-15
        match code {
            30..=37 => code - 30,        // Standard foreground color
            40..=47 => code - 40,        // Standard background color
            90..=97 => code - 90 + 8,    // Bright foreground color
            100..=107 => code - 100 + 8, // Bright background color
            _ => return None,
        }
    } else {
        return None;
    };

    Some(if is_background {
        apply_background_color(text, color_code)
    } else {
        apply_foreground_color(text, color_code)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ansi_color_to_egui() {
        // Test standard colors
        assert_eq!(ansi_color_to_egui(0), Color32::BLACK);
        assert_eq!(ansi_color_to_egui(1), Color32::RED);
        assert_eq!(ansi_color_to_egui(2), Color32::GREEN);

        // Test bright colors
        assert_eq!(ansi_color_to_egui(8), Color32::from_rgb(128, 128, 128));
        assert_eq!(ansi_color_to_egui(9), Color32::from_rgb(255, 128, 128));
    }

    #[test]
    fn test_parse_4bit_color() {
        // Test foreground color
        let _red_text = parse_4bit_color("Hello", "31", false).unwrap();
        // Test background color
        let _red_bg_text = parse_4bit_color("Hello", "41", true).unwrap();
        // Test bright color
        let _bright_red_text = parse_4bit_color("Hello", "91", false).unwrap();

        // These tests just ensure the function does not return None; actual color values need to be verified by other means
        assert!(parse_4bit_color("Hello", "31", false).is_some());
        assert!(parse_4bit_color("Hello", "41", true).is_some());
        assert!(parse_4bit_color("Hello", "91", false).is_some());
    }
}
