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
