use eframe::{App, Frame, egui};
use egui::{Color32, RichText, Sense, Vec2};
use egui_sgr::{
    AnsiColor, AnsiSpan, AnsiSpanBuffer, EguiAnsiTheme, ansi_to_spans, spans_to_layout_job,
};

const PRESETS: &[Preset] = &[
    Preset {
        name: "4-bit",
        input: "Normal \x1b[31mred\x1b[0m, \x1b[34mblue\x1b[0m, \x1b[93mbright yellow\x1b[0m.",
    },
    Preset {
        name: "256 color",
        input: "\x1b[38;5;208morange\x1b[0m, \x1b[38;5;51msky blue\x1b[0m, \x1b[48;5;21;38;5;15mwhite on blue\x1b[0m.",
    },
    Preset {
        name: "Truecolor",
        input: "\x1b[38;2;255;105;180mhot pink\x1b[0m and \x1b[48;2;30;30;30;38;2;180;220;255msoft blue on dark\x1b[0m.",
    },
    Preset {
        name: "Attributes",
        input: "\x1b[1;31mBold red\x1b[0m \x1b[3;32mItalic green\x1b[0m \x1b[4;58;5;208mUnderlined\x1b[0m \x1b[9mStrike\x1b[0m \x1b[7mReverse\x1b[0m",
    },
    Preset {
        name: "Colon RGB",
        input: "\x1b[38:2::255:105:180mColon truecolor\x1b[0m and \x1b[4:3;58:5:51mcurly semantic underline\x1b[0m.",
    },
    Preset {
        name: "Terminal log",
        input: "\x1b[1;31merror\x1b[0m: file not found\n\x1b[33mwarning\x1b[0m: using fallback\n\x1b[32mdone\x1b[0m in 42ms",
    },
];

const STREAM: &[&str] = &[
    "\x1b[32mstream ",
    "keeps ",
    "style ",
    "across ",
    "chunks\x1b[0m, ",
    "\x1b[38;5;208mthen orange\x1b[0m.",
];

#[derive(Debug, Clone, Copy)]
struct Preset {
    name: &'static str,
    input: &'static str,
}

struct AnsiColorDemo {
    selected_preset: usize,
    custom_input: String,
    stream_buffer: AnsiSpanBuffer,
    stream_cursor: usize,
}

impl Default for AnsiColorDemo {
    fn default() -> Self {
        Self {
            selected_preset: 3,
            custom_input: display_ansi_sequence(PRESETS[3].input),
            stream_buffer: AnsiSpanBuffer::new(),
            stream_cursor: 0,
        }
    }
}

impl App for AnsiColorDemo {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut Frame) {
        ui.heading("egui_sgr");
        ui.horizontal_wrapped(|ui| {
            ui.label("ANSI/SGR");
            ui.separator();
            ui.label("semantic spans");
            ui.separator();
            ui.label("egui LayoutJob");
            ui.separator();
            ui.label("stream-safe bytes");
        });
        ui.separator();

        let theme = EguiAnsiTheme::default();
        let parsed_input = parse_user_input(&self.custom_input);
        let spans = ansi_to_spans(&parsed_input);
        let job = spans_to_layout_job(&spans, &theme);

        ui.columns(2, |columns| {
            columns[0].set_min_width(320.0);
            self.show_controls(&mut columns[0], &parsed_input, &spans);

            columns[1].set_min_width(440.0);
            self.show_preview(&mut columns[1], &theme, &parsed_input, &spans, job);
        });
    }
}

impl AnsiColorDemo {
    fn show_controls(&mut self, ui: &mut egui::Ui, parsed_input: &str, spans: &[AnsiSpan]) {
        ui.heading("Input");
        ui.horizontal_wrapped(|ui| {
            for (index, preset) in PRESETS.iter().enumerate() {
                if ui
                    .selectable_label(self.selected_preset == index, preset.name)
                    .clicked()
                {
                    self.selected_preset = index;
                    self.custom_input = display_ansi_sequence(preset.input);
                }
            }
        });

        ui.add_space(8.0);
        ui.add(
            egui::TextEdit::multiline(&mut self.custom_input)
                .code_editor()
                .desired_rows(7)
                .lock_focus(true),
        );

        ui.horizontal_wrapped(|ui| {
            if ui.button("Red").clicked() {
                self.custom_input.push_str("\\x1b[31mRed\\x1b[0m ");
            }
            if ui.button("Orange").clicked() {
                self.custom_input.push_str("\\x1b[38;5;208mOrange\\x1b[0m ");
            }
            if ui.button("RGB").clicked() {
                self.custom_input
                    .push_str("\\x1b[38;2;255;105;180mPink\\x1b[0m ");
            }
            if ui.button("Underline").clicked() {
                self.custom_input
                    .push_str("\\x1b[4;58;5;51mUnderline\\x1b[0m ");
            }
            if ui.button("Reverse").clicked() {
                self.custom_input.push_str("\\x1b[7mReverse\\x1b[0m ");
            }
        });

        ui.add_space(10.0);
        self.show_metrics(ui, parsed_input, spans);
        ui.add_space(10.0);
        self.show_streaming_demo(ui);
    }

    fn show_metrics(&self, ui: &mut egui::Ui, parsed_input: &str, spans: &[AnsiSpan]) {
        ui.heading("Metrics");

        egui::Grid::new("metrics_grid")
            .num_columns(2)
            .spacing([16.0, 4.0])
            .show(ui, |ui| {
                ui.label("Bytes");
                ui.monospace(parsed_input.len().to_string());
                ui.end_row();

                ui.label("Visible chars");
                ui.monospace(
                    spans
                        .iter()
                        .map(|span| span.text.chars().count())
                        .sum::<usize>()
                        .to_string(),
                );
                ui.end_row();

                ui.label("Spans");
                ui.monospace(spans.len().to_string());
                ui.end_row();
            });
    }

    fn show_streaming_demo(&mut self, ui: &mut egui::Ui) {
        ui.heading("Stream");

        ui.horizontal(|ui| {
            if ui.button("Push").clicked()
                && let Some(chunk) = STREAM.get(self.stream_cursor)
            {
                self.stream_buffer.push_str(chunk);
                self.stream_cursor += 1;
            }
            if ui.button("Finish").clicked() {
                self.stream_buffer.finish();
            }
            if ui.button("Reset").clicked() {
                self.stream_buffer.clear();
                self.stream_cursor = 0;
            }
        });

        ui.horizontal_wrapped(|ui| {
            for (index, chunk) in STREAM.iter().enumerate() {
                let text = display_ansi_sequence(chunk);
                if index < self.stream_cursor {
                    ui.label(RichText::new(text).monospace().strong());
                } else {
                    ui.label(RichText::new(text).monospace().weak());
                }
            }
        });

        let theme = EguiAnsiTheme::default();
        ui.label(self.stream_buffer.to_layout_job(&theme));
    }

    fn show_preview(
        &self,
        ui: &mut egui::Ui,
        theme: &EguiAnsiTheme,
        parsed_input: &str,
        spans: &[AnsiSpan],
        job: egui::text::LayoutJob,
    ) {
        ui.heading("LayoutJob");
        ui.label(job);

        ui.add_space(8.0);
        ui.collapsing("Escaped input", |ui| {
            ui.monospace(display_ansi_sequence(parsed_input));
        });

        ui.add_space(8.0);
        self.show_span_table(ui, spans);

        ui.add_space(8.0);
        self.show_palette(ui, theme);
    }

    fn show_span_table(&self, ui: &mut egui::Ui, spans: &[AnsiSpan]) {
        ui.heading("AnsiSpan");

        egui::Grid::new("span_grid")
            .num_columns(5)
            .striped(true)
            .spacing([12.0, 4.0])
            .show(ui, |ui| {
                ui.strong("#");
                ui.strong("Text");
                ui.strong("FG");
                ui.strong("BG");
                ui.strong("Style");
                ui.end_row();

                for (index, span) in spans.iter().enumerate() {
                    ui.monospace(index.to_string());
                    ui.monospace(visible_debug_text(&span.text));
                    ui.monospace(color_label(span.style.foreground));
                    ui.monospace(color_label(span.style.background));
                    ui.monospace(style_label(span));
                    ui.end_row();
                }
            });
    }

    fn show_palette(&self, ui: &mut egui::Ui, theme: &EguiAnsiTheme) {
        ui.heading("Palette");

        egui::Grid::new("palette_grid")
            .num_columns(16)
            .spacing([3.0, 3.0])
            .show(ui, |ui| {
                for index in 0..32 {
                    palette_swatch(ui, index as u8, theme.palette[index]);
                    if index % 16 == 15 {
                        ui.end_row();
                    }
                }
            });
    }
}

fn display_ansi_sequence(input: &str) -> String {
    input.replace('\x1b', "\\x1b")
}

fn parse_user_input(input: &str) -> String {
    input
        .replace("\\x1b", "\x1b")
        .replace("\\x1B", "\x1b")
        .replace("\\X1b", "\x1b")
        .replace("\\X1B", "\x1b")
        .replace("\\033", "\x1b")
}

fn visible_debug_text(text: &str) -> String {
    text.replace('\n', "\\n").replace('\r', "\\r")
}

fn color_label(color: AnsiColor) -> String {
    match color {
        AnsiColor::Default => "default".to_owned(),
        AnsiColor::Indexed(index) => format!("idx {index}"),
        AnsiColor::Rgb(r, g, b) => format!("rgb({r},{g},{b})"),
    }
}

fn style_label(span: &AnsiSpan) -> String {
    let style = span.style;
    let mut parts = Vec::new();

    if style.intensity != Default::default() {
        parts.push(format!("{:?}", style.intensity).to_lowercase());
    }
    if style.italic {
        parts.push("italic".to_owned());
    }
    if style.underline != Default::default() {
        parts.push(format!("{:?}", style.underline).to_lowercase());
    }
    if style.strikethrough {
        parts.push("strike".to_owned());
    }
    if style.reverse {
        parts.push("reverse".to_owned());
    }
    if style.hidden {
        parts.push("hidden".to_owned());
    }
    if let Some(color) = style.underline_color {
        parts.push(format!("underline {}", color_label(color)));
    }

    if parts.is_empty() {
        "normal".to_owned()
    } else {
        parts.join(", ")
    }
}

fn palette_swatch(ui: &mut egui::Ui, index: u8, color: Color32) {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(18.0), Sense::hover());
    ui.painter().rect_filled(rect, 2.0, color);
    response.on_hover_text(format!(
        "{index}: #{:02X}{:02X}{:02X}",
        color.r(),
        color.g(),
        color.b()
    ));
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1040.0, 760.0]),
        ..Default::default()
    };

    eframe::run_native(
        "egui_sgr Demo",
        options,
        Box::new(|_cc| Ok(Box::new(AnsiColorDemo::default()))),
    )
}
