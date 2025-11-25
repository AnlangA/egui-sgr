use egui::{Color32, RichText};

/// Converts RGB values to an egui Color32.
///
/// This function handles 24-bit true color mode which allows
/// direct specification of RGB values (0-255 each component),
/// providing access to the full 16.77 million color palette.
///
/// # Arguments
/// - `r`: Red component (0-255)
/// - `g`: Green component (0-255)
/// - `b`: Blue component (0-255)
///
/// # Returns
/// The corresponding Color32 color
pub fn rgb_to_egui(r: u8, g: u8, b: u8) -> Color32 {
    Color32::from_rgb(r, g, b)
}

/// Applies a foreground color
pub fn apply_foreground_color(text: &str, r: u8, g: u8, b: u8) -> RichText {
    let color = rgb_to_egui(r, g, b);
    RichText::new(text).color(color)
}

/// Applies a background color
pub fn apply_background_color(text: &str, r: u8, g: u8, b: u8) -> RichText {
    let color = rgb_to_egui(r, g, b);
    RichText::new(text).background_color(color)
}

/// Parses a 24-bit true color ANSI sequence and applies the color
///
/// # Arguments
/// - `text`: The text to render
/// - `sequence`: The ANSI color sequence, e.g., "2;255;105;180" (2 for true color mode, the next three are RGB values)
/// - `is_background`: Whether it is a background color
///
/// # Returns
/// RichText with the color applied
pub fn parse_24bit_color(text: &str, sequence: &str, is_background: bool) -> Option<RichText> {
    // Matches 24-bit true color sequences of the format: 2;<r>;<g>;<b>
    let re = regex::Regex::new(r"^2;(\d+);(\d+);(\d+)$").ok()?;

    let caps = re.captures(sequence)?;
    let r_str = caps.get(1)?.as_str();
    let g_str = caps.get(2)?.as_str();
    let b_str = caps.get(3)?.as_str();

    if let (Ok(r), Ok(g), Ok(b)) = (
        r_str.parse::<u8>(),
        g_str.parse::<u8>(),
        b_str.parse::<u8>(),
    ) {
        Some(if is_background {
            apply_background_color(text, r, g, b)
        } else {
            apply_foreground_color(text, r, g, b)
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb_to_egui() {
        // Test basic colors
        assert_eq!(rgb_to_egui(255, 0, 0), Color32::from_rgb(255, 0, 0)); // Red
        assert_eq!(rgb_to_egui(0, 255, 0), Color32::from_rgb(0, 255, 0)); // Green
        assert_eq!(rgb_to_egui(0, 0, 255), Color32::from_rgb(0, 0, 255)); // Blue

        // Test other colors
        assert_eq!(rgb_to_egui(255, 105, 180), Color32::from_rgb(255, 105, 180)); // Hot Pink
        assert_eq!(rgb_to_egui(128, 128, 128), Color32::from_rgb(128, 128, 128)); // Gray
    }

    #[test]
    fn test_parse_24bit_color() {
        // Test basic colors
        assert!(parse_24bit_color("Hello", "2;255;0;0", false).is_some()); // Red foreground
        assert!(parse_24bit_color("Hello", "2;0;255;0", true).is_some()); // Green background

        // Test other colors
        assert!(parse_24bit_color("Hello", "2;255;105;180", false).is_some()); // Hot pink foreground
        assert!(parse_24bit_color("Hello", "2;30;30;30", true).is_some()); // Dark gray background

        // Test boundary values
        assert!(parse_24bit_color("Hello", "2;0;0;0", false).is_some()); // Black
        assert!(parse_24bit_color("Hello", "2;255;255;255", false).is_some()); // White

        // Test invalid values
        assert!(parse_24bit_color("Hello", "2;256;0;0", false).is_none()); // R value out of range
        assert!(parse_24bit_color("Hello", "2;0;256;0", false).is_none()); // G value out of range
        assert!(parse_24bit_color("Hello", "2;0;0;256", false).is_none()); // B value out of range
        assert!(parse_24bit_color("Hello", "5;255;0;0", false).is_none()); // Incorrect color mode
    }
}
