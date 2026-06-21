use eframe::{App, Frame, egui};
use egui_sgr::{AnsiParser, AnsiSpanBuffer, EguiAnsiTheme, ansi_to_layout_job};

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

struct AnsiColorDemo {
    examples: Vec<(&'static str, &'static str)>,
    custom_input: String,
    stream_buffer: AnsiSpanBuffer,
    stream_cursor: usize,
}

impl Default for AnsiColorDemo {
    fn default() -> Self {
        Self {
            examples: vec![
                (
                    "4-bit",
                    "This is \x1b[31mred\x1b[0m, \x1b[34mblue\x1b[0m and \x1b[93mbright yellow\x1b[0m.",
                ),
                (
                    "8-bit",
                    "This is \x1b[38;5;208morange\x1b[0m, \x1b[38;5;51msky blue\x1b[0m and \x1b[48;5;21mblue background\x1b[0m.",
                ),
                (
                    "24-bit",
                    "This is \x1b[38;2;255;105;180mhot pink\x1b[0m and \x1b[48;2;30;30;30mtrue-color background\x1b[0m.",
                ),
                (
                    "Attributes",
                    "\x1b[1;31mBold red\x1b[0m \x1b[3;32mItalic green\x1b[0m \x1b[4;58;5;208mUnderlined\x1b[0m \x1b[7mReverse\x1b[0m",
                ),
                (
                    "Colon RGB",
                    "\x1b[38:2::255:105:180mColon truecolor\x1b[0m and \x1b[4:3mcurly-style semantic underline\x1b[0m.",
                ),
            ],
            custom_input:
                "\\x1b[1;31mBold red\\x1b[0m \\x1b[38;5;208mOrange\\x1b[0m \\x1b[7mReverse\\x1b[0m"
                    .to_string(),
            stream_buffer: AnsiSpanBuffer::new(),
            stream_cursor: 0,
        }
    }
}

impl App for AnsiColorDemo {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut Frame) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.heading("egui_sgr Demo");
            ui.separator();

            egui::ScrollArea::vertical()
                .id_salt("demo_scroll")
                .show(ui, |ui| {
                    let theme = EguiAnsiTheme::default();

                    for (name, input) in &self.examples {
                        ui.heading(*name);
                        ui.monospace(display_ansi_sequence(input));
                        ui.label(ansi_to_layout_job(input, &theme));
                        ui.add_space(14.0);
                    }

                    self.show_custom_input(ui, &theme);
                    ui.add_space(14.0);
                    self.show_streaming_demo(ui, &theme);
                });
        });
    }
}

impl AnsiColorDemo {
    fn show_custom_input(&mut self, ui: &mut egui::Ui, theme: &EguiAnsiTheme) {
        ui.heading("Custom input");
        ui.text_edit_multiline(&mut self.custom_input);

        ui.horizontal_wrapped(|ui| {
            if ui.button("Red").clicked() {
                self.custom_input.push_str("\\x1b[31mRed\\x1b[0m ");
            }
            if ui.button("Orange").clicked() {
                self.custom_input.push_str("\\x1b[38;5;208mOrange\\x1b[0m ");
            }
            if ui.button("Underline").clicked() {
                self.custom_input
                    .push_str("\\x1b[4;58;5;51mUnderline\\x1b[0m ");
            }
        });

        let parsed_input = parse_user_input(&self.custom_input);
        ui.label(ansi_to_layout_job(&parsed_input, theme));

        let mut parser = AnsiParser::new();
        let colored_segments = parser.parse(&parsed_input);
        ui.collapsing("Legacy ColoredText segments", |ui| {
            egui::Grid::new("segments_grid")
                .num_columns(2)
                .show(ui, |ui| {
                    ui.label("Text");
                    ui.label("Colors");
                    ui.end_row();

                    for segment in colored_segments {
                        ui.label(segment.text);
                        ui.monospace(format!(
                            "fg={:?}, bg={:?}",
                            segment.foreground_color, segment.background_color
                        ));
                        ui.end_row();
                    }
                });
        });
    }

    fn show_streaming_demo(&mut self, ui: &mut egui::Ui, theme: &EguiAnsiTheme) {
        const STREAM: &[&str] = &[
            "\x1b[32mstream ",
            "keeps ",
            "green\x1b[0m, ",
            "\x1b[38;5;208mthen ",
            "orange\x1b[0m.",
        ];

        ui.heading("Streaming input");
        ui.horizontal(|ui| {
            if ui.button("Push chunk").clicked()
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

        ui.label(self.stream_buffer.to_layout_job(theme));
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([900.0, 700.0]),
        ..Default::default()
    };

    eframe::run_native(
        "egui_sgr Demo",
        options,
        Box::new(|_cc| Ok(Box::new(AnsiColorDemo::default()))),
    )
}
