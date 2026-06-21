use eframe::{App, Frame, egui};
use egui_sgr::{EguiAnsiTheme, ansi_to_layout_job};

struct LayoutJobExample {
    input: String,
}

impl Default for LayoutJobExample {
    fn default() -> Self {
        Self {
            input: "normal \\x1b[31mred\\x1b[0m and \\x1b[38;5;208morange\\x1b[0m".to_owned(),
        }
    }
}

impl App for LayoutJobExample {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut Frame) {
        ui.heading("ansi_to_layout_job");

        ui.add(
            egui::TextEdit::multiline(&mut self.input)
                .code_editor()
                .desired_rows(3),
        );

        ui.separator();

        let theme = EguiAnsiTheme::default();
        let ansi = parse_escaped_ansi(&self.input);
        let job = ansi_to_layout_job(&ansi, &theme);

        ui.label(job.clone());

        ui.add_space(8.0);
        egui::Grid::new("layout_job_sections")
            .num_columns(2)
            .striped(true)
            .spacing([16.0, 4.0])
            .show(ui, |ui| {
                ui.strong("Bytes");
                ui.strong("Color");
                ui.end_row();

                for section in &job.sections {
                    ui.monospace(format!("{:?}", section.byte_range));
                    ui.monospace(format!("{:?}", section.format.color));
                    ui.end_row();
                }
            });
    }
}

fn parse_escaped_ansi(input: &str) -> String {
    input
        .replace("\\x1b", "\x1b")
        .replace("\\x1B", "\x1b")
        .replace("\\033", "\x1b")
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([520.0, 320.0]),
        ..Default::default()
    };

    eframe::run_native(
        "egui_sgr layout_job",
        options,
        Box::new(|_cc| Ok(Box::new(LayoutJobExample::default()))),
    )
}
