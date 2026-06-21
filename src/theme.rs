use egui::{Color32, TextFormat};

/// Theme used when converting ANSI spans into egui text formats.
#[derive(Debug, Clone, PartialEq)]
pub struct EguiAnsiTheme {
    /// Base egui text format copied before ANSI-specific fields are applied.
    pub default_format: TextFormat,
    /// Foreground color used when ANSI foreground is [`AnsiColor::Default`](crate::AnsiColor::Default).
    pub default_foreground: Color32,
    /// Background color used for reverse video when ANSI background is default.
    pub default_background: Color32,
    /// ANSI 0-255 color palette.
    pub palette: [Color32; 256],
    /// Width used for underlines in egui strokes.
    pub underline_width: f32,
    /// Width used for strikethrough strokes.
    pub strikethrough_width: f32,
    /// Alpha multiplier applied to faint text.
    pub faint_opacity: f32,
    /// Whether bold 0-7 indexed foreground colors render as bright 8-15 colors.
    pub bold_is_bright: bool,
}

impl Default for EguiAnsiTheme {
    fn default() -> Self {
        let default_foreground = Color32::from_rgb(229, 229, 229);
        let default_format = TextFormat {
            color: default_foreground,
            background: Color32::TRANSPARENT,
            ..Default::default()
        };

        Self {
            default_format,
            default_foreground,
            default_background: Color32::BLACK,
            palette: Self::xterm_palette(),
            underline_width: 1.0,
            strikethrough_width: 1.0,
            faint_opacity: 0.6,
            bold_is_bright: true,
        }
    }
}

impl EguiAnsiTheme {
    /// Returns a theme using a conventional xterm 256-color palette.
    #[must_use]
    pub fn xterm() -> Self {
        Self::default()
    }

    /// Returns a theme matching the 0.1.x compatibility palette as closely as
    /// the span model allows.
    #[must_use]
    pub fn legacy() -> Self {
        let default_foreground = Color32::WHITE;
        let default_format = TextFormat {
            color: default_foreground,
            background: Color32::TRANSPARENT,
            ..Default::default()
        };

        Self {
            default_format,
            default_foreground,
            default_background: Color32::BLACK,
            palette: Self::legacy_palette(),
            underline_width: 1.0,
            strikethrough_width: 1.0,
            faint_opacity: 0.6,
            bold_is_bright: false,
        }
    }

    /// Builds the xterm 256-color palette.
    #[must_use]
    pub fn xterm_palette() -> [Color32; 256] {
        build_palette([
            Color32::from_rgb(0, 0, 0),
            Color32::from_rgb(205, 0, 0),
            Color32::from_rgb(0, 205, 0),
            Color32::from_rgb(205, 205, 0),
            Color32::from_rgb(0, 0, 238),
            Color32::from_rgb(205, 0, 205),
            Color32::from_rgb(0, 205, 205),
            Color32::from_rgb(229, 229, 229),
            Color32::from_rgb(127, 127, 127),
            Color32::from_rgb(255, 0, 0),
            Color32::from_rgb(0, 255, 0),
            Color32::from_rgb(255, 255, 0),
            Color32::from_rgb(92, 92, 255),
            Color32::from_rgb(255, 0, 255),
            Color32::from_rgb(0, 255, 255),
            Color32::from_rgb(255, 255, 255),
        ])
    }

    /// Builds the palette used by the 0.1.x API compatibility path.
    #[must_use]
    pub fn legacy_palette() -> [Color32; 256] {
        build_palette([
            Color32::BLACK,
            Color32::RED,
            Color32::GREEN,
            Color32::YELLOW,
            Color32::BLUE,
            Color32::from_rgb(255, 0, 255),
            Color32::from_rgb(0, 255, 255),
            Color32::from_rgb(255, 255, 255),
            Color32::from_rgb(128, 128, 128),
            Color32::from_rgb(255, 128, 128),
            Color32::from_rgb(128, 255, 128),
            Color32::from_rgb(255, 255, 128),
            Color32::from_rgb(128, 128, 255),
            Color32::from_rgb(255, 128, 255),
            Color32::from_rgb(128, 255, 255),
            Color32::WHITE,
        ])
    }
}

fn build_palette(system_colors: [Color32; 16]) -> [Color32; 256] {
    let mut palette = [Color32::BLACK; 256];

    palette[..16].copy_from_slice(&system_colors);

    for code in 16u8..=231 {
        let value = code - 16;
        let r = value / 36;
        let g = (value % 36) / 6;
        let b = value % 6;
        palette[code as usize] =
            Color32::from_rgb(cube_component(r), cube_component(g), cube_component(b));
    }

    for code in 232u8..=255 {
        let gray = 8 + (code - 232) * 10;
        palette[code as usize] = Color32::from_rgb(gray, gray, gray);
    }

    palette
}

fn cube_component(component: u8) -> u8 {
    if component == 0 {
        0
    } else {
        55 + component * 40
    }
}
