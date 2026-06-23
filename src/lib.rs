#![forbid(unsafe_code)]
#![warn(missing_docs, rustdoc::broken_intra_doc_links)]

//! Convert ANSI/SGR text into egui text representations.
//!
//! The primary output is [`egui::text::LayoutJob`], because ANSI text is a
//! single logical string with style changes inside it. Compatibility helpers
//! for [`egui::RichText`] and the older `ColoredText` model are still
//! available behind the `legacy` feature.
//!
//! ```rust
//! use egui_sgr::{ansi_to_layout_job, EguiAnsiTheme};
//!
//! let theme = EguiAnsiTheme::default();
//! let job = ansi_to_layout_job("\x1b[31mred\x1b[0m default", &theme);
//! assert_eq!(job.text, "red default");
//! ```

mod egui_render;
mod model;
mod parser;
mod sgr;
mod theme;

/// Legacy color conversion helpers.
#[cfg(feature = "legacy")]
pub mod color_models;

#[cfg(feature = "legacy")]
pub use color_models::*;
pub use egui_render::{ansi_bytes_to_layout_job, ansi_to_layout_job, spans_to_layout_job};
#[cfg(feature = "legacy")]
pub use egui_render::{
    ansi_to_rich_text, ansi_to_rich_text_with_theme, convert_to_rich_text, spans_to_rich_text,
};
#[cfg(feature = "legacy")]
pub use model::ColoredText;
pub use model::{AnsiColor, AnsiIntensity, AnsiSpan, AnsiStyle, UnderlineStyle};
pub use parser::{AnsiSpanBuffer, AnsiStreamParser, ansi_bytes_to_spans, ansi_to_spans};
pub use theme::EguiAnsiTheme;

/// Compatibility parser that returns the legacy foreground/background model.
///
/// This parser intentionally keeps the historical one-shot behavior: every call
/// to [`AnsiParser::parse`] starts from the default style. Use
/// [`AnsiStreamParser`] when ANSI state must survive across chunks.
#[derive(Debug, Default, Clone)]
#[cfg(feature = "legacy")]
pub struct AnsiParser;

#[cfg(feature = "legacy")]
impl AnsiParser {
    /// Creates a new one-shot compatibility parser.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Parses ANSI/SGR text into the legacy [`ColoredText`] representation.
    #[must_use]
    pub fn parse(&mut self, input: &str) -> Vec<ColoredText> {
        if input.is_empty() {
            return vec![ColoredText::new("")];
        }

        let spans = ansi_to_spans(input);
        egui_render::spans_to_colored_text(&spans, &EguiAnsiTheme::legacy())
    }
}

/// Small compile-checked usage sample used by examples and documentation.
pub fn example_usage() {
    let theme = EguiAnsiTheme::default();
    let _job = ansi_to_layout_job("\x1b[38;5;208morange\x1b[0m", &theme);
    #[cfg(feature = "legacy")]
    let _rich_text = ansi_to_rich_text("\x1b[31mred\x1b[0m");
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "legacy")]
    use egui::Color32;
    use egui::Stroke;

    fn text_of(spans: &[AnsiSpan]) -> String {
        spans.iter().map(|span| span.text.as_str()).collect()
    }

    #[test]
    fn ansi_to_spans_parses_basic_4bit_color() {
        let spans = ansi_to_spans("\x1b[31mRed\x1b[0m Default");

        assert_eq!(spans.len(), 2);
        assert_eq!(spans[0].text, "Red");
        assert_eq!(spans[0].style.foreground, AnsiColor::Indexed(1));
        assert_eq!(spans[1].text, " Default");
        assert_eq!(spans[1].style.foreground, AnsiColor::Default);
    }

    #[test]
    fn ansi_to_spans_parses_8bit_and_24bit_colors() {
        let spans = ansi_to_spans("\x1b[38;5;208mOrange\x1b[48;2;1;2;3mBg");

        assert_eq!(spans.len(), 2);
        assert_eq!(spans[0].style.foreground, AnsiColor::Indexed(208));
        assert_eq!(spans[1].style.foreground, AnsiColor::Indexed(208));
        assert_eq!(spans[1].style.background, AnsiColor::Rgb(1, 2, 3));
    }

    #[test]
    fn ansi_to_spans_supports_colon_truecolor_forms() {
        let spans = ansi_to_spans("\x1b[38:2::255:105:180mPink");

        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].style.foreground, AnsiColor::Rgb(255, 105, 180));
    }

    #[test]
    fn ansi_to_spans_tracks_text_attributes() {
        let spans = ansi_to_spans("\x1b[1;3;4:3;9mStyled\x1b[22;23;24;29mPlain");

        assert_eq!(spans[0].style.intensity, AnsiIntensity::Bold);
        assert!(spans[0].style.italic);
        assert_eq!(spans[0].style.underline, UnderlineStyle::Curly);
        assert!(spans[0].style.strikethrough);
        assert_eq!(spans[1].style.intensity, AnsiIntensity::Normal);
        assert!(!spans[1].style.italic);
        assert_eq!(spans[1].style.underline, UnderlineStyle::None);
        assert!(!spans[1].style.strikethrough);
    }

    #[test]
    fn stream_parser_keeps_style_across_chunks() {
        let mut parser = AnsiStreamParser::new();
        let first = parser.push_bytes(b"\x1b[31mRe");
        let second = parser.push_bytes(b"d");
        let third = parser.push_bytes(b"\x1b[0m Plain");

        assert_eq!(text_of(&first), "Re");
        assert_eq!(first[0].style.foreground, AnsiColor::Indexed(1));
        assert_eq!(text_of(&second), "d");
        assert_eq!(second[0].style.foreground, AnsiColor::Indexed(1));
        assert_eq!(text_of(&third), " Plain");
        assert_eq!(third[0].style.foreground, AnsiColor::Default);
    }

    #[test]
    fn stream_parser_handles_split_escape_sequence() {
        let mut parser = AnsiStreamParser::new();

        assert!(parser.push_bytes(b"\x1b").is_empty());
        assert!(parser.push_bytes(b"[38;5;").is_empty());
        let spans = parser.push_bytes(b"208mOrange");

        assert_eq!(text_of(&spans), "Orange");
        assert_eq!(spans[0].style.foreground, AnsiColor::Indexed(208));
    }

    #[test]
    fn stream_parser_handles_split_utf8() {
        let mut parser = AnsiStreamParser::new();
        let bytes = "你".as_bytes();

        assert!(parser.push_bytes(&bytes[..1]).is_empty());
        assert!(parser.push_bytes(&bytes[1..2]).is_empty());
        let spans = parser.push_bytes(&bytes[2..]);

        assert_eq!(text_of(&spans), "你");
    }

    #[test]
    fn stream_parser_drops_unfinished_escape_on_finish() {
        let mut parser = AnsiStreamParser::new();

        let spans = parser.push_bytes(b"Start\x1b[31");
        let tail = parser.finish();

        assert_eq!(text_of(&spans), "Start");
        assert!(tail.is_empty());
        assert_eq!(parser.current_style(), &AnsiStyle::default());
    }

    #[test]
    fn span_buffer_merges_adjacent_same_style_chunks() {
        let mut buffer = AnsiSpanBuffer::new();

        buffer.push_bytes(b"\x1b[32mHel");
        buffer.push_bytes(b"lo");

        assert_eq!(buffer.spans().len(), 1);
        assert_eq!(buffer.spans()[0].text, "Hello");
        assert_eq!(buffer.spans()[0].style.foreground, AnsiColor::Indexed(2));
    }

    #[test]
    fn layout_job_contains_expected_sections() {
        let theme = EguiAnsiTheme::default();
        let job = ansi_to_layout_job("A\x1b[31mRed\x1b[0mZ", &theme);

        assert_eq!(job.text, "ARedZ");
        assert_eq!(job.sections.len(), 3);
        assert_eq!(job.sections[1].byte_range, 1..4);
        assert_eq!(job.sections[1].format.color, theme.palette[1]);
    }

    #[test]
    fn ansi_to_spans_merges_redundant_same_style_runs() {
        let spans = ansi_to_spans("\x1b[31mA\x1b[31mB");

        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].text, "AB");
        assert_eq!(spans[0].style.foreground, AnsiColor::Indexed(1));
    }

    #[test]
    fn layout_job_merges_redundant_same_style_sections() {
        let theme = EguiAnsiTheme::default();
        let job = ansi_to_layout_job("\x1b[31mA\x1b[31mB", &theme);

        assert_eq!(job.text, "AB");
        assert_eq!(job.sections.len(), 1);
        assert_eq!(job.sections[0].byte_range, 0..2);
        assert_eq!(job.sections[0].format.color, theme.palette[1]);
    }

    #[test]
    fn direct_layout_job_matches_span_rendering() {
        let theme = EguiAnsiTheme::default();
        let input =
            "\x1b[1;31mError\x1b[0m \x1b[38;5;208mwarning\x1b[0m \x1b[48;2;1;2;3;38:2::4:5:6mnote";
        let spans = ansi_to_spans(input);

        assert_eq!(
            ansi_to_layout_job(input, &theme),
            spans_to_layout_job(&spans, &theme)
        );
    }

    #[test]
    fn layout_job_maps_background_underline_and_strikethrough() {
        let theme = EguiAnsiTheme::default();
        let job = ansi_to_layout_job("\x1b[48;5;21;58;5;196;4;9mText", &theme);
        let format = &job.sections[0].format;

        assert_eq!(format.background, theme.palette[21]);
        assert_eq!(
            format.underline,
            Stroke::new(theme.underline_width, theme.palette[196])
        );
        assert_eq!(
            format.strikethrough,
            Stroke::new(theme.strikethrough_width, format.color)
        );
    }

    #[test]
    fn reverse_video_is_render_time_style() {
        let spans = ansi_to_spans("\x1b[31;42;7mSwap");
        let theme = EguiAnsiTheme::default();
        let job = spans_to_layout_job(&spans, &theme);
        let format = &job.sections[0].format;

        assert_eq!(spans[0].style.foreground, AnsiColor::Indexed(1));
        assert_eq!(spans[0].style.background, AnsiColor::Indexed(2));
        assert_eq!(format.color, theme.palette[2]);
        assert_eq!(format.background, theme.palette[1]);
    }

    #[test]
    #[cfg(feature = "legacy")]
    fn ansi_parser_compatibility_is_one_shot() {
        let mut parser = AnsiParser::new();

        let red = parser.parse("\x1b[31mRed");
        let plain = parser.parse("Plain");

        assert_eq!(red[0].text, "Red");
        assert_eq!(red[0].foreground_color, Some(Color32::RED));
        assert_eq!(plain[0].text, "Plain");
        assert_eq!(plain[0].foreground_color, None);
    }

    #[test]
    #[cfg(feature = "legacy")]
    fn ansi_parser_compatibility_keeps_empty_input_segment() {
        let mut parser = AnsiParser::new();
        let segments = parser.parse("");

        assert_eq!(segments, vec![ColoredText::new("")]);
    }

    #[test]
    #[cfg(feature = "legacy")]
    fn rich_text_compatibility_keeps_empty_input_segment() {
        let rich_text = ansi_to_rich_text("");

        assert_eq!(rich_text.len(), 1);
        assert_eq!(rich_text[0].text(), "");
    }

    #[test]
    fn non_sgr_sequences_are_stripped() {
        let spans = ansi_to_spans("Before\x1b]0;Title\x07\x1b[2JAfter");

        assert_eq!(text_of(&spans), "BeforeAfter");
    }

    #[test]
    #[cfg(feature = "legacy")]
    fn escaped_backslash_sequences_are_plain_text() {
        let mut parser = AnsiParser::new();
        let input = "\\x1b[31mRed\\x1b[0m";
        let segments = parser.parse(input);

        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].text, input);
        assert_eq!(segments[0].foreground_color, None);
    }

    #[test]
    #[cfg(feature = "legacy")]
    fn colored_text_constructors_work() {
        let plain = ColoredText::new("Hello");
        let fg = ColoredText::with_foreground("Hello", Color32::RED);
        let bg = ColoredText::with_background("Hello", Color32::BLUE);
        let both = ColoredText::with_colors("Hello", Some(Color32::RED), Some(Color32::BLUE));

        assert_eq!(plain.foreground_color, None);
        assert_eq!(fg.foreground_color, Some(Color32::RED));
        assert_eq!(bg.background_color, Some(Color32::BLUE));
        assert_eq!(both.foreground_color, Some(Color32::RED));
        assert_eq!(both.background_color, Some(Color32::BLUE));
    }

    #[test]
    #[cfg(feature = "legacy")]
    fn mixed_colors_keep_expected_text_segments() {
        let result =
            ansi_to_rich_text("Normal \x1b[31mred\x1b[0m normal \x1b[38;5;208morange\x1b[0m done");

        assert_eq!(result.len(), 5);
        assert_eq!(result[0].text(), "Normal ");
        assert_eq!(result[1].text(), "red");
        assert_eq!(result[2].text(), " normal ");
        assert_eq!(result[3].text(), "orange");
        assert_eq!(result[4].text(), " done");
    }

    #[test]
    #[cfg(feature = "legacy")]
    fn background_color_is_preserved_in_legacy_parser() {
        let mut parser = AnsiParser::new();
        let segments = parser.parse("\x1b[41mWhite on red\x1b[0m");

        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].text, "White on red");
        assert_eq!(segments[0].background_color, Some(Color32::RED));
    }

    #[test]
    #[cfg(feature = "legacy")]
    fn foreground_and_background_color_are_preserved() {
        let mut parser = AnsiParser::new();
        let segments = parser.parse("\x1b[31;43mRed on yellow\x1b[0m");

        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].foreground_color, Some(Color32::RED));
        assert_eq!(segments[0].background_color, Some(Color32::YELLOW));
    }

    #[test]
    #[cfg(feature = "legacy")]
    fn sequential_color_changes_split_segments() {
        let mut parser = AnsiParser::new();
        let segments = parser
            .parse("Default\x1b[31mRed\x1b[43mRed on yellow\x1b[32mGreen on yellow\x1b[0mDefault");

        let texts: Vec<_> = segments
            .iter()
            .map(|segment| segment.text.as_str())
            .collect();
        assert_eq!(
            texts,
            vec![
                "Default",
                "Red",
                "Red on yellow",
                "Green on yellow",
                "Default"
            ]
        );
        assert_eq!(segments[0].foreground_color, None);
        assert_eq!(segments[1].foreground_color, Some(Color32::RED));
        assert_eq!(segments[2].background_color, Some(Color32::YELLOW));
        assert_eq!(segments[4].foreground_color, None);
    }

    #[test]
    #[cfg(feature = "legacy")]
    fn reset_foreground_color() {
        let mut parser = AnsiParser::new();
        let segments = parser.parse("\x1b[31mRed\x1b[39mDefault");

        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].foreground_color, Some(Color32::RED));
        assert_eq!(segments[1].foreground_color, None);
    }

    #[test]
    #[cfg(feature = "legacy")]
    fn reset_background_color() {
        let mut parser = AnsiParser::new();
        let segments = parser.parse("\x1b[41mRed BG\x1b[49mDefault BG");

        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].background_color, Some(Color32::RED));
        assert_eq!(segments[1].background_color, None);
    }

    #[test]
    fn bright_foreground_colors_are_indexed() {
        let spans = ansi_to_spans("\x1b[90mBright Black\x1b[91mBright Red\x1b[97mBright White");

        assert_eq!(spans.len(), 3);
        assert_eq!(spans[0].style.foreground, AnsiColor::Indexed(8));
        assert_eq!(spans[1].style.foreground, AnsiColor::Indexed(9));
        assert_eq!(spans[2].style.foreground, AnsiColor::Indexed(15));
    }

    #[test]
    fn bright_background_colors_are_indexed() {
        let spans =
            ansi_to_spans("\x1b[100mBright Black BG\x1b[101mBright Red BG\x1b[107mBright White BG");

        assert_eq!(spans.len(), 3);
        assert_eq!(spans[0].style.background, AnsiColor::Indexed(8));
        assert_eq!(spans[1].style.background, AnsiColor::Indexed(9));
        assert_eq!(spans[2].style.background, AnsiColor::Indexed(15));
    }

    #[test]
    fn eight_bit_background_color() {
        let spans = ansi_to_spans("\x1b[48;5;196mRed BG");

        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].style.background, AnsiColor::Indexed(196));
    }

    #[test]
    #[cfg(feature = "legacy")]
    fn truecolor_foreground_value() {
        let mut parser = AnsiParser::new();
        let segments = parser.parse("\x1b[38;2;255;0;0mRed\x1b[0m");

        assert_eq!(
            segments[0].foreground_color,
            Some(Color32::from_rgb(255, 0, 0))
        );
    }

    #[test]
    #[cfg(feature = "legacy")]
    fn truecolor_background_value() {
        let mut parser = AnsiParser::new();
        let segments = parser.parse("\x1b[48;2;0;255;0mGreen BG\x1b[0m");

        assert_eq!(
            segments[0].background_color,
            Some(Color32::from_rgb(0, 255, 0))
        );
    }

    #[test]
    fn indexed_color_boundary_values() {
        let spans = ansi_to_spans(
            "\x1b[38;5;0mA\x1b[38;5;15mB\x1b[38;5;16mC\x1b[38;5;231mD\x1b[38;5;232mE\x1b[38;5;255mF",
        );

        assert_eq!(spans.len(), 6);
        assert_eq!(spans[0].style.foreground, AnsiColor::Indexed(0));
        assert_eq!(spans[5].style.foreground, AnsiColor::Indexed(255));
    }

    #[test]
    #[cfg(feature = "legacy")]
    fn consecutive_resets_are_harmless() {
        let mut parser = AnsiParser::new();
        let segments = parser.parse("\x1b[0m\x1b[0m\x1b[0mText\x1b[0m");

        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0], ColoredText::new("Text"));
    }

    #[test]
    #[cfg(feature = "legacy")]
    fn default_parser_works() {
        let mut parser: AnsiParser = Default::default();
        let segments = parser.parse("\x1b[31mRed\x1b[0m");

        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].text, "Red");
    }

    #[test]
    #[cfg(feature = "legacy")]
    fn multiline_text_keeps_newline() {
        let mut parser = AnsiParser::new();
        let segments = parser.parse("\x1b[31mLine1\nLine2\x1b[0m");

        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].text, "Line1\nLine2");
    }

    #[test]
    #[cfg(feature = "legacy")]
    fn unicode_text_is_preserved() {
        let mut parser = AnsiParser::new();
        let segments = parser.parse("\x1b[31m你好世界\x1b[0m");

        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].text, "你好世界");
    }

    #[test]
    #[cfg(feature = "legacy")]
    fn empty_reset_sequence_resets_style() {
        let mut parser = AnsiParser::new();
        let segments = parser.parse("\x1b[31mRed\x1b[mDefault");

        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].foreground_color, Some(Color32::RED));
        assert_eq!(segments[1].foreground_color, None);
    }

    #[test]
    fn linux_prompt_format_is_linearized() {
        let spans = ansi_to_spans("\x1b[1;31mroot\x1b[m@\x1b[1;34mhost\x1b[m:#");

        assert_eq!(text_of(&spans), "root@host:#");
    }

    #[test]
    fn bash_prompt_colors_are_linearized() {
        let spans = ansi_to_spans("\x1b[1;34muser@host\x1b[m:\x1b[1;32m~\x1b[m$ ");

        assert_eq!(text_of(&spans), "user@host:~$ ");
    }

    #[test]
    fn cursor_movement_csi_is_ignored() {
        let spans = ansi_to_spans("\x1b[2J\x1b[H\x1b[31mRed\x1b[0m");

        assert_eq!(text_of(&spans), "Red");
    }

    #[test]
    fn complex_terminal_output_keeps_visible_text() {
        let spans = ansi_to_spans(
            "\x1b[1;31mError:\x1b[m \x1b[33mFile not found\x1b[m\n\x1b[32mDone\x1b[m",
        );

        assert_eq!(text_of(&spans), "Error: File not found\nDone");
    }

    #[test]
    fn text_attributes_reset_independently() {
        let spans = ansi_to_spans("\x1b[1mBold\x1b[22m\x1b[4mUnderline\x1b[24mNormal");

        assert_eq!(text_of(&spans), "BoldUnderlineNormal");
        assert_eq!(spans[0].style.intensity, AnsiIntensity::Bold);
        assert_eq!(spans[1].style.underline, UnderlineStyle::Single);
        assert_eq!(spans[2].style.underline, UnderlineStyle::None);
    }

    #[test]
    fn ansi_bytes_to_layout_job_matches_string_api() {
        let theme = EguiAnsiTheme::default();
        let input = "\x1b[38;5;208mOrange\x1b[0m";

        assert_eq!(
            ansi_bytes_to_layout_job(input.as_bytes(), &theme),
            ansi_to_layout_job(input, &theme)
        );
    }

    #[test]
    fn invalid_utf8_is_replaced_without_panicking() {
        let spans = ansi_bytes_to_spans(b"ok \xFF done");

        assert_eq!(text_of(&spans), "ok \u{FFFD} done");
    }

    #[test]
    fn unfinished_utf8_is_dropped_at_finish() {
        let mut parser = AnsiStreamParser::new();

        assert!(parser.push_bytes(&[0xE4]).is_empty());
        assert!(parser.finish().is_empty());
    }

    #[test]
    fn osc_st_sequence_is_stripped() {
        let spans = ansi_to_spans("Before\x1b]0;Title\x1b\\After");

        assert_eq!(text_of(&spans), "BeforeAfter");
    }

    #[test]
    fn malformed_truecolor_is_ignored_without_style_leak() {
        let spans = ansi_to_spans("\x1b[38;2;300;0;0mPlain");

        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].text, "Plain");
        assert_eq!(spans[0].style.foreground, AnsiColor::Default);
    }

    #[test]
    fn underline_color_reset_keeps_underline_style() {
        let spans = ansi_to_spans("\x1b[4;58;5;196mA\x1b[59mB");

        assert_eq!(spans.len(), 2);
        assert_eq!(spans[0].style.underline, UnderlineStyle::Single);
        assert_eq!(
            spans[0].style.underline_color,
            Some(AnsiColor::Indexed(196))
        );
        assert_eq!(spans[1].style.underline, UnderlineStyle::Single);
        assert_eq!(spans[1].style.underline_color, None);
    }

    #[test]
    fn reverse_reset_restores_normal_rendering() {
        let spans = ansi_to_spans("\x1b[31;42;7mA\x1b[27mB");
        let theme = EguiAnsiTheme::default();
        let job = spans_to_layout_job(&spans, &theme);

        assert_eq!(job.sections[0].format.color, theme.palette[2]);
        assert_eq!(job.sections[0].format.background, theme.palette[1]);
        assert_eq!(job.sections[1].format.color, theme.palette[1]);
        assert_eq!(job.sections[1].format.background, theme.palette[2]);
    }

    #[test]
    fn default_theme_renders_bold_low_colors_as_bright() {
        let theme = EguiAnsiTheme::default();
        let job = ansi_to_layout_job("\x1b[1;31mBold red", &theme);

        assert_eq!(job.sections[0].format.color, theme.palette[9]);
    }

    #[test]
    fn legacy_theme_does_not_brighten_bold_low_colors() {
        let theme = EguiAnsiTheme::legacy();
        let job = ansi_to_layout_job("\x1b[1;31mBold red", &theme);

        assert_eq!(job.sections[0].format.color, theme.palette[1]);
    }

    #[test]
    fn span_buffer_clear_resets_spans_and_parser_state() {
        let mut buffer = AnsiSpanBuffer::new();

        buffer.push_bytes(b"\x1b[31mRed");
        buffer.clear();
        buffer.push_bytes(b"Plain");

        assert_eq!(buffer.spans().len(), 1);
        assert_eq!(buffer.spans()[0].text, "Plain");
        assert_eq!(buffer.spans()[0].style.foreground, AnsiColor::Default);
    }
}
