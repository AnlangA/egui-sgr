use eframe::{App, Frame, egui};
use egui_sgr::{AnsiColor, AnsiSpanBuffer, EguiAnsiTheme};

const CHUNKS: &[&str] = &[
    "\x1b[32mstream ",
    "keeps ",
    "green\x1b[0m ",
    "\x1b[38;5;208mthen orange\x1b[0m",
];

struct StreamingExample {
    buffer: AnsiSpanBuffer,
    next_chunk: usize,
}

impl Default for StreamingExample {
    fn default() -> Self {
        let mut example = Self {
            buffer: AnsiSpanBuffer::new(),
            next_chunk: 0,
        };
        example.push_next_chunk();
        example.push_next_chunk();
        example
    }
}

impl App for StreamingExample {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut Frame) {
        ui.heading("AnsiSpanBuffer");

        ui.horizontal(|ui| {
            if ui.button("Push").clicked() {
                self.push_next_chunk();
            }
            if ui.button("Finish").clicked() {
                self.buffer.finish();
            }
            if ui.button("Reset").clicked() {
                self.buffer.clear();
                self.next_chunk = 0;
            }
        });

        ui.horizontal_wrapped(|ui| {
            for (index, chunk) in CHUNKS.iter().enumerate() {
                let text = display_ansi_sequence(chunk);
                if index < self.next_chunk {
                    ui.monospace(text);
                } else {
                    ui.label(egui::RichText::new(text).monospace().weak());
                }
            }
        });

        ui.separator();

        let theme = EguiAnsiTheme::default();
        ui.label(self.buffer.to_layout_job(&theme));

        ui.add_space(8.0);
        egui::Grid::new("streamed_spans")
            .num_columns(2)
            .striped(true)
            .spacing([16.0, 4.0])
            .show(ui, |ui| {
                ui.strong("Color");
                ui.strong("Text");
                ui.end_row();

                for span in self.buffer.spans() {
                    ui.monospace(color_label(span.style.foreground));
                    ui.monospace(&span.text);
                    ui.end_row();
                }
            });
    }
}

impl StreamingExample {
    fn push_next_chunk(&mut self) {
        if let Some(chunk) = CHUNKS.get(self.next_chunk) {
            self.buffer.push_str(chunk);
            self.next_chunk += 1;
        }
    }
}

fn display_ansi_sequence(input: &str) -> String {
    input.replace('\x1b', "\\x1b")
}

fn color_label(color: AnsiColor) -> String {
    match color {
        AnsiColor::Default => "default".to_owned(),
        AnsiColor::Indexed(index) => format!("idx {index}"),
        AnsiColor::Rgb(r, g, b) => format!("rgb({r},{g},{b})"),
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([560.0, 340.0]),
        ..Default::default()
    };

    eframe::run_native(
        "egui_sgr streaming",
        options,
        Box::new(|_cc| Ok(Box::new(StreamingExample::default()))),
    )
}
