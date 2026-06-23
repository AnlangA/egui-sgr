#[cfg(feature = "legacy")]
use egui::Color32;

/// ANSI color representation before it is mapped into an egui color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AnsiColor {
    /// Use the theme's default color.
    #[default]
    Default,
    /// Use a color from the 0-255 ANSI palette.
    Indexed(u8),
    /// Use an explicit RGB color.
    Rgb(u8, u8, u8),
}

/// ANSI text intensity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AnsiIntensity {
    /// Normal intensity.
    #[default]
    Normal,
    /// Bold/intense text.
    Bold,
    /// Faint/dim text.
    Faint,
}

/// ANSI underline style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum UnderlineStyle {
    /// No underline.
    #[default]
    None,
    /// Single underline.
    Single,
    /// Double underline.
    Double,
    /// Curly underline semantic style.
    Curly,
    /// Dotted underline semantic style.
    Dotted,
    /// Dashed underline semantic style.
    Dashed,
}

/// Complete style state for an ANSI span.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct AnsiStyle {
    /// Foreground color.
    pub foreground: AnsiColor,
    /// Background color.
    pub background: AnsiColor,
    /// Optional underline color.
    pub underline_color: Option<AnsiColor>,
    /// Text intensity.
    pub intensity: AnsiIntensity,
    /// Whether italic text is active.
    pub italic: bool,
    /// Active underline style.
    pub underline: UnderlineStyle,
    /// Whether strikethrough is active.
    pub strikethrough: bool,
    /// Whether reverse video is active.
    pub reverse: bool,
    /// Whether hidden/conceal text is active.
    pub hidden: bool,
}

impl AnsiStyle {
    pub(crate) fn reset(&mut self) {
        *self = Self::default();
    }
}

/// A visible run of text with one ANSI style.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AnsiSpan {
    /// Visible text for this span.
    pub text: String,
    /// ANSI style for this span.
    pub style: AnsiStyle,
}

impl AnsiSpan {
    /// Creates a new ANSI span.
    #[must_use]
    pub fn new(text: impl Into<String>, style: AnsiStyle) -> Self {
        Self {
            text: text.into(),
            style,
        }
    }
}

/// Legacy text segment with only foreground/background egui colors.
#[cfg(feature = "legacy")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColoredText {
    /// The text content of this segment.
    pub text: String,
    /// Optional foreground color.
    pub foreground_color: Option<Color32>,
    /// Optional background color.
    pub background_color: Option<Color32>,
}

#[cfg(feature = "legacy")]
impl ColoredText {
    /// Creates a segment with no colors applied.
    #[must_use]
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            foreground_color: None,
            background_color: None,
        }
    }

    /// Creates a segment with a foreground color.
    #[must_use]
    pub fn with_foreground(text: impl Into<String>, color: Color32) -> Self {
        Self {
            text: text.into(),
            foreground_color: Some(color),
            background_color: None,
        }
    }

    /// Creates a segment with a background color.
    #[must_use]
    pub fn with_background(text: impl Into<String>, color: Color32) -> Self {
        Self {
            text: text.into(),
            foreground_color: None,
            background_color: Some(color),
        }
    }

    /// Creates a segment with optional foreground and background colors.
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
