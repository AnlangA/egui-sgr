use crate::{AnsiColor, AnsiIntensity, AnsiSpan, AnsiStyle, EguiAnsiTheme, UnderlineStyle, sgr};
use egui::text::{LayoutJob, LayoutSection};
use egui::{Color32, Stroke, TextFormat};
use vte::{Params, Perform};

/// Converts ANSI spans to an egui layout job.
#[must_use]
pub fn spans_to_layout_job(spans: &[AnsiSpan], theme: &EguiAnsiTheme) -> LayoutJob {
    let mut job = LayoutJob::default();
    job.text
        .reserve(spans.iter().map(|span| span.text.len()).sum());
    job.sections.reserve(spans.len());
    let mut last_style = None;

    for span in spans {
        append_styled_text(&mut job, &span.text, span.style, theme, &mut last_style);
    }

    job
}

/// Converts a UTF-8 string with ANSI escapes directly to an egui layout job.
#[must_use]
pub fn ansi_to_layout_job(input: &str, theme: &EguiAnsiTheme) -> LayoutJob {
    ansi_bytes_to_layout_job(input.as_bytes(), theme)
}

/// Converts bytes with ANSI escapes directly to an egui layout job.
#[must_use]
pub fn ansi_bytes_to_layout_job(input: &[u8], theme: &EguiAnsiTheme) -> LayoutJob {
    let mut parser = vte::Parser::new();
    let mut performer = LayoutJobPerformer::new(theme, input.len());
    parser.advance(&mut performer, input);
    performer.finish()
}

struct LayoutJobPerformer<'a> {
    theme: &'a EguiAnsiTheme,
    current_style: AnsiStyle,
    text: String,
    job: LayoutJob,
    last_style: Option<AnsiStyle>,
}

impl<'a> LayoutJobPerformer<'a> {
    fn new(theme: &'a EguiAnsiTheme, input_len: usize) -> Self {
        let mut job = LayoutJob::default();
        job.text.reserve(input_len);

        Self {
            theme,
            current_style: AnsiStyle::default(),
            text: String::new(),
            job,
            last_style: None,
        }
    }

    fn flush_text(&mut self) {
        if self.text.is_empty() {
            return;
        }

        let text = std::mem::take(&mut self.text);
        append_styled_text(
            &mut self.job,
            &text,
            self.current_style,
            self.theme,
            &mut self.last_style,
        );
    }

    fn finish(mut self) -> LayoutJob {
        self.flush_text();
        self.job
    }
}

impl Perform for LayoutJobPerformer<'_> {
    fn print(&mut self, c: char) {
        self.text.push(c);
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            b'\n' => self.text.push('\n'),
            b'\r' => self.text.push('\r'),
            b'\t' => self.text.push('\t'),
            _ => {}
        }
    }

    fn csi_dispatch(&mut self, params: &Params, intermediates: &[u8], ignore: bool, action: char) {
        if action == 'm' && intermediates.is_empty() && !ignore {
            self.flush_text();
            sgr::apply_sgr(params, &mut self.current_style);
        }
    }
}

fn append_styled_text(
    job: &mut LayoutJob,
    text: &str,
    style: AnsiStyle,
    theme: &EguiAnsiTheme,
    last_style: &mut Option<AnsiStyle>,
) {
    if text.is_empty() {
        return;
    }

    let start = job.text.len();
    job.text.push_str(text);
    let end = job.text.len();

    if *last_style == Some(style)
        && let Some(section) = job.sections.last_mut()
        && section.byte_range.end == start
    {
        section.byte_range.end = end;
        return;
    }

    job.sections.push(LayoutSection {
        leading_space: 0.0,
        byte_range: start..end,
        format: text_format_for_style(&style, theme),
    });
    *last_style = Some(style);
}

fn text_format_for_style(style: &AnsiStyle, theme: &EguiAnsiTheme) -> TextFormat {
    let colors = effective_colors(style, theme);
    let mut format = theme.default_format.clone();

    format.color = colors.foreground;
    format.background = colors.background.unwrap_or(theme.default_format.background);
    format.italics = style.italic;
    format.underline = if style.underline == UnderlineStyle::None {
        Stroke::NONE
    } else {
        let underline_color = style
            .underline_color
            .map(|color| resolve_color(color, theme))
            .unwrap_or(colors.foreground);
        Stroke::new(theme.underline_width, underline_color)
    };
    format.strikethrough = if style.strikethrough {
        Stroke::new(theme.strikethrough_width, colors.foreground)
    } else {
        Stroke::NONE
    };

    format
}

#[derive(Debug, Clone, Copy)]
struct EffectiveColors {
    foreground: Color32,
    background: Option<Color32>,
}

fn effective_colors(style: &AnsiStyle, theme: &EguiAnsiTheme) -> EffectiveColors {
    let mut foreground = foreground_color(style, theme);
    let mut background = background_color(style, theme);

    if style.reverse {
        let original_foreground = foreground;
        foreground = background.unwrap_or(theme.default_background);
        background = Some(original_foreground);
    }

    if style.hidden {
        foreground = Color32::TRANSPARENT;
    } else if style.intensity == AnsiIntensity::Faint {
        foreground = with_scaled_alpha(foreground, theme.faint_opacity);
    }

    EffectiveColors {
        foreground,
        background,
    }
}

fn foreground_color(style: &AnsiStyle, theme: &EguiAnsiTheme) -> Color32 {
    match style.foreground {
        AnsiColor::Indexed(index)
            if theme.bold_is_bright && style.intensity == AnsiIntensity::Bold && index < 8 =>
        {
            theme.palette[(index + 8) as usize]
        }
        color => resolve_color_or_default(color, theme.default_foreground, theme),
    }
}

fn background_color(style: &AnsiStyle, theme: &EguiAnsiTheme) -> Option<Color32> {
    match style.background {
        AnsiColor::Default => None,
        color => Some(resolve_color(color, theme)),
    }
}

fn resolve_color_or_default(
    color: AnsiColor,
    default_color: Color32,
    theme: &EguiAnsiTheme,
) -> Color32 {
    match color {
        AnsiColor::Default => default_color,
        color => resolve_color(color, theme),
    }
}

fn resolve_color(color: AnsiColor, theme: &EguiAnsiTheme) -> Color32 {
    match color {
        AnsiColor::Default => theme.default_foreground,
        AnsiColor::Indexed(index) => theme.palette[index as usize],
        AnsiColor::Rgb(r, g, b) => Color32::from_rgb(r, g, b),
    }
}

fn with_scaled_alpha(color: Color32, opacity: f32) -> Color32 {
    let alpha = ((color.a() as f32) * opacity.clamp(0.0, 1.0)).round() as u8;
    Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha)
}
