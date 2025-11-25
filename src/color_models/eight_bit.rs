use egui::{Color32, RichText};

/// Converts an 8-bit color code (0-255) to an egui Color32.
///
/// The 256-color palette is divided into:
/// - 0-15: Standard 16 colors (same as 4-bit)
/// - 16-231: 6×6×6 RGB color cube
/// - 232-255: 24 levels of grayscale
///
/// # Arguments
/// - `color_code`: A color code from 0-255
///
/// # Returns
/// The corresponding Color32 color
pub fn ansi_256_to_egui(color_code: u8) -> Color32 {
    match color_code {
        // 0-15: Standard 16 colors (same as 4-bit)
        0 => Color32::BLACK,
        1 => Color32::RED,
        2 => Color32::GREEN,
        3 => Color32::YELLOW,
        4 => Color32::BLUE,
        5 => Color32::from_rgb(255, 0, 255),    // Magenta
        6 => Color32::from_rgb(0, 255, 255),    // Cyan
        7 => Color32::from_rgb(192, 192, 192),  // White (slightly darker)
        8 => Color32::from_rgb(128, 128, 128),  // Bright Black/Gray
        9 => Color32::from_rgb(255, 128, 128),  // Bright Red
        10 => Color32::from_rgb(128, 255, 128), // Bright Green
        11 => Color32::from_rgb(255, 255, 128), // Bright Yellow
        12 => Color32::from_rgb(128, 128, 255), // Bright Blue
        13 => Color32::from_rgb(255, 128, 255), // Bright Magenta
        14 => Color32::from_rgb(128, 255, 255), // Bright Cyan
        15 => Color32::WHITE,

        // 16-231: 6x6x6 RGB cube
        16..=231 => {
            // Calculate RGB values
            let code = color_code - 16;
            let r = code / 36;          // Red component
            let g = (code % 36) / 6;    // Green component
            let b = code % 6;           // Blue component

            // Convert values from 0-5 to the 0-255 range
            // The formula for ANSI 256-color cube components is: 40c + 55 (for c=1..4), 0 (for c=0), 255 (for c=5)
            let convert_component = |c: u8| {
                if c == 0 { 0 } else { 55 + c * 40 }
            };

            Color32::from_rgb(
                convert_component(r),
                convert_component(g),
                convert_component(b),
            )
        }

        // 232-255: 24 levels of grayscale
        232..=255 => {
            // Grayscale from black to white, in steps of 10, with a special case for 255
            let gray = if color_code == 255 {
                248 // Special case for pure white
            } else {
                8 + (color_code - 232) * 10
            };
            Color32::from_rgb(gray, gray, gray)
        }
    }
}

/// Applies a foreground color
pub fn apply_foreground_color(text: &str, color_code: u8) -> RichText {
    let color = ansi_256_to_egui(color_code);
    RichText::new(text).color(color)
}

/// Applies a background color
pub fn apply_background_color(text: &str, color_code: u8) -> RichText {
    let color = ansi_256_to_egui(color_code);
    RichText::new(text).background_color(color)
}

/// Parses an 8-bit color ANSI sequence and applies the color
///
/// # Arguments
/// - `text`: The text to render
/// - `sequence`: The ANSI color sequence, e.g., "5;208" (5 for 256-color mode, 208 is the color code)
/// - `is_background`: Whether it is a background color
///
/// # Returns
/// RichText with the color applied
pub fn parse_8bit_color(text: &str, sequence: &str, is_background: bool) -> Option<RichText> {
    // Matches 8-bit color sequences of the format: 5;<n> where n is 0-255
    let re = regex::Regex::new(r"^5;(\d+)$").ok()?;

    let caps = re.captures(sequence)?;
    let color_str = caps.get(1)?.as_str();

    if let Ok(color_code) = color_str.parse::<u8>() {
        Some(if is_background {
            apply_background_color(text, color_code)
        } else {
            apply_foreground_color(text, color_code)
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ansi_256_to_egui() {
        // Test standard colors
        assert_eq!(ansi_256_to_egui(0), Color32::BLACK);
        assert_eq!(ansi_256_to_egui(1), Color32::RED);
        assert_eq!(ansi_256_to_egui(2), Color32::GREEN);
        assert_eq!(ansi_256_to_egui(15), Color32::WHITE);

        // Test RGB cube
        // 16 is black
        assert_eq!(ansi_256_to_egui(16), Color32::from_rgb(0, 0, 0));

        // 17 is the darkest blue (r=0, g=0, b=1 => 55+1*40=95)
        assert_eq!(ansi_256_to_egui(17), Color32::from_rgb(0, 0, 95));

        // 52 is the darkest red (r=1, g=0, b=0 => 55+1*40=95)
        assert_eq!(ansi_256_to_egui(52), Color32::from_rgb(95, 0, 0));

        // 22 is the darkest green (r=0, g=1, b=0 => 55+1*40=95)
        assert_eq!(ansi_256_to_egui(22), Color32::from_rgb(0, 95, 0));

        // Test grayscale
        // 232 is a dark gray close to black
        assert_eq!(ansi_256_to_egui(232), Color32::from_rgb(8, 8, 8));
        // 255 is white
        assert_eq!(ansi_256_to_egui(255), Color32::from_rgb(248, 248, 248));
    }

    #[test]
    fn test_parse_8bit_color() {
        // Test standard colors
        assert!(parse_8bit_color("Hello", "5;1", false).is_some());
        assert!(parse_8bit_color("Hello", "5;15", false).is_some());

        // Test RGB cube
        assert!(parse_8bit_color("Hello", "5;208", false).is_some());
        assert!(parse_8bit_color("Hello", "5;208", true).is_some());

        // Test grayscale
        assert!(parse_8bit_color("Hello", "5;240", false).is_some());

        // Test invalid values
        assert!(parse_8bit_color("Hello", "5;256", false).is_none());
        assert!(parse_8bit_color("Hello", "4;1", false).is_none());
    }
}
