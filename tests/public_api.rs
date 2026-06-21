use egui::Color32;
use egui_sgr::{
    AnsiColor, AnsiParser, AnsiSpanBuffer, AnsiStreamParser, EguiAnsiTheme, ansi_to_layout_job,
    ansi_to_rich_text, ansi_to_spans, spans_to_layout_job,
};

#[test]
fn public_layout_job_flow() {
    let theme = EguiAnsiTheme::default();
    let spans = ansi_to_spans("plain \x1b[38;5;208morange\x1b[0m");
    let job = spans_to_layout_job(&spans, &theme);

    assert_eq!(job.text, "plain orange");
    assert_eq!(job.sections[1].format.color, theme.palette[208]);
    assert_eq!(
        job,
        ansi_to_layout_job("plain \x1b[38;5;208morange\x1b[0m", &theme)
    );
}

#[test]
fn public_stream_parser_flow() {
    let mut parser = AnsiStreamParser::new();

    assert!(parser.push_bytes(b"\x1b[31").is_empty());
    let spans = parser.push_bytes(b"mred");

    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].text, "red");
    assert_eq!(spans[0].style.foreground, AnsiColor::Indexed(1));
    assert_eq!(parser.current_style().foreground, AnsiColor::Indexed(1));
}

#[test]
fn public_span_buffer_flow() {
    let theme = EguiAnsiTheme::default();
    let mut buffer = AnsiSpanBuffer::new();

    buffer.push_str("\x1b[32mgreen ");
    buffer.push_str("text\x1b[0m");
    buffer.finish();

    assert_eq!(buffer.spans().len(), 1);
    assert_eq!(buffer.to_layout_job(&theme).text, "green text");
}

#[test]
fn public_legacy_flow() {
    let mut parser = AnsiParser::new();
    let segments = parser.parse("\x1b[31mred\x1b[0m");
    let rich_text = ansi_to_rich_text("\x1b[31mred\x1b[0m");

    assert_eq!(segments.len(), 1);
    assert_eq!(segments[0].foreground_color, Some(Color32::RED));
    assert_eq!(rich_text.len(), 1);
    assert_eq!(rich_text[0].text(), "red");
}
