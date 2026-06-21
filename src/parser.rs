use crate::{AnsiSpan, AnsiStyle, EguiAnsiTheme, sgr};
use egui::text::LayoutJob;
use vte::{Params, Perform};

/// Stateful streaming ANSI parser.
///
/// Feed process output, PTY output, network chunks, or any other byte stream
/// with [`Self::push_bytes`]. Style state and incomplete ANSI/UTF-8 sequences
/// are preserved between calls.
pub struct AnsiStreamParser {
    parser: vte::Parser,
    performer: SgrPerformer,
}

impl Default for AnsiStreamParser {
    fn default() -> Self {
        Self::new()
    }
}

impl AnsiStreamParser {
    /// Creates a new streaming parser.
    #[must_use]
    pub fn new() -> Self {
        Self {
            parser: vte::Parser::new(),
            performer: SgrPerformer::new(),
        }
    }

    /// Pushes a byte chunk and returns visible spans produced by this chunk.
    #[must_use]
    pub fn push_bytes(&mut self, chunk: &[u8]) -> Vec<AnsiSpan> {
        self.parser.advance(&mut self.performer, chunk);
        self.performer.flush_text();
        self.performer.take_output()
    }

    /// Pushes a UTF-8 string chunk.
    #[must_use]
    pub fn push_str(&mut self, chunk: &str) -> Vec<AnsiSpan> {
        self.push_bytes(chunk.as_bytes())
    }

    /// Finishes the current stream and resets parser state.
    ///
    /// Unfinished escape, OSC, DCS, or UTF-8 sequences are discarded.
    #[must_use]
    pub fn finish(&mut self) -> Vec<AnsiSpan> {
        self.performer.flush_text();
        let output = self.performer.take_output();
        self.reset();
        output
    }

    /// Clears all parser and style state.
    pub fn reset(&mut self) {
        self.parser = vte::Parser::new();
        self.performer = SgrPerformer::new();
    }

    /// Returns the currently active ANSI style.
    #[must_use]
    pub fn current_style(&self) -> &AnsiStyle {
        &self.performer.current_style
    }
}

/// Accumulates streamed ANSI spans and can render the full buffer to egui.
pub struct AnsiSpanBuffer {
    parser: AnsiStreamParser,
    spans: Vec<AnsiSpan>,
}

impl Default for AnsiSpanBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl AnsiSpanBuffer {
    /// Creates an empty span buffer.
    #[must_use]
    pub fn new() -> Self {
        Self {
            parser: AnsiStreamParser::new(),
            spans: Vec::new(),
        }
    }

    /// Pushes a byte chunk into the buffer.
    pub fn push_bytes(&mut self, chunk: &[u8]) {
        let spans = self.parser.push_bytes(chunk);
        extend_and_merge(&mut self.spans, spans);
    }

    /// Pushes a UTF-8 string chunk into the buffer.
    pub fn push_str(&mut self, chunk: &str) {
        self.push_bytes(chunk.as_bytes());
    }

    /// Finishes the stream and stores any final visible spans.
    pub fn finish(&mut self) {
        let spans = self.parser.finish();
        extend_and_merge(&mut self.spans, spans);
    }

    /// Returns all accumulated spans.
    #[must_use]
    pub fn spans(&self) -> &[AnsiSpan] {
        &self.spans
    }

    /// Clears accumulated spans and parser state.
    pub fn clear(&mut self) {
        self.spans.clear();
        self.parser.reset();
    }

    /// Converts the accumulated spans to an egui layout job.
    #[must_use]
    pub fn to_layout_job(&self, theme: &EguiAnsiTheme) -> LayoutJob {
        crate::spans_to_layout_job(&self.spans, theme)
    }
}

/// Converts a UTF-8 string into ANSI spans.
#[must_use]
pub fn ansi_to_spans(input: &str) -> Vec<AnsiSpan> {
    ansi_bytes_to_spans(input.as_bytes())
}

/// Converts bytes into ANSI spans.
#[must_use]
pub fn ansi_bytes_to_spans(input: &[u8]) -> Vec<AnsiSpan> {
    let mut parser = AnsiStreamParser::new();
    let mut spans = parser.push_bytes(input);
    spans.extend(parser.finish());
    spans
}

fn extend_and_merge(target: &mut Vec<AnsiSpan>, spans: Vec<AnsiSpan>) {
    for span in spans {
        if span.text.is_empty() {
            continue;
        }

        if let Some(last) = target.last_mut()
            && last.style == span.style
        {
            last.text.push_str(&span.text);
            continue;
        }

        target.push(span);
    }
}

struct SgrPerformer {
    current_style: AnsiStyle,
    text: String,
    output: Vec<AnsiSpan>,
}

impl SgrPerformer {
    fn new() -> Self {
        Self {
            current_style: AnsiStyle::default(),
            text: String::new(),
            output: Vec::new(),
        }
    }

    fn flush_text(&mut self) {
        if self.text.is_empty() {
            return;
        }

        self.output.push(AnsiSpan::new(
            std::mem::take(&mut self.text),
            self.current_style,
        ));
    }

    fn take_output(&mut self) -> Vec<AnsiSpan> {
        std::mem::take(&mut self.output)
    }
}

impl Perform for SgrPerformer {
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
