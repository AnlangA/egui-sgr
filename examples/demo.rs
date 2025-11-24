use eframe::{App, Frame, egui};
use egui_sgr::{AnsiParser, ansi_to_rich_text};

// Helper function to display ANSI escape sequences in a readable format
fn display_ansi_sequence(input: &str) -> String {
    input.replace("\x1b", "\\x1b")
}

// Helper function to convert user input with \x1b string to actual control character
fn parse_ansi_input(input: &str) -> String {
    input.replace("\\x1b", "\x1b")
}

struct AnsiColorDemo {
    input_4bit: String,
    input_8bit: String,
    input_24bit: String,
    input_mixed: String,
    custom_input: String,
}

impl Default for AnsiColorDemo {
    fn default() -> Self {
        Self {
            input_4bit: "This is \x1b[31mred\x1b[0m, \x1b[34mblue\x1b[0m and \x1b[33myellow\x1b[0m text".to_string(),
            input_8bit: "This is \x1b[38;5;208morange\x1b[0m, \x1b[38;5;51msky blue\x1b[0m and \x1b[38;5;120mgreen\x1b[0m text".to_string(),
            input_24bit: "This is \x1b[38;2;255;105;180mhot pink\x1b[0m and \x1b[38;2;128;0;128mpurple\x1b[0m text".to_string(),
            input_mixed: "Normal text \x1b[31mred\x1b[0m normal \x1b[38;5;208morange\x1b[0m normal \x1b[38;2;255;105;180mpink\x1b[0m normal".to_string(),
            custom_input: "\\x1b[1;31mBold red\\x1b[0m \\x1b[4;32mUnderline green\\x1b[0m \\x1b[7;33mInverted yellow\\x1b[0m".to_string(),
        }
    }
}

impl App for AnsiColorDemo {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("egui_sgr Demo - ANSI Escape Sequence Color Model Conversion");
            ui.separator();

            egui::ScrollArea::vertical()
                .id_salt("demo_scroll")
                .show(ui, |ui| {
                    // 4-bit color example
                    self.show_4bit_example(ui);
                    ui.add_space(20.0);

                    // 8-bit color example
                    self.show_8bit_example(ui);
                    ui.add_space(20.0);

                    // 24-bit color example
                    self.show_24bit_example(ui);
                    ui.add_space(20.0);

                    // Mixed color example
                    self.show_mixed_example(ui);
                    ui.add_space(20.0);

                    // Custom input
                    self.show_custom_input(ui);
                });
        });
    }
}

impl AnsiColorDemo {
    fn show_4bit_example(&self, ui: &mut egui::Ui) {
        ui.heading("4-bit Color (16 Colors) Example");

        ui.horizontal(|ui| {
            ui.label("ANSI Sequence:");
            ui.monospace(display_ansi_sequence(&self.input_4bit));
        });

        ui.label("Conversion Result:");
        let rich_text = ansi_to_rich_text(&self.input_4bit);
        ui.horizontal_wrapped(|ui| {
            for text in rich_text {
                ui.label(text);
            }
        });

        ui.add_space(10.0);

        // Show all 16 colors
        ui.label("All 16 Colors:");
        ui.horizontal_wrapped(|ui| {
            let colors = [
                ("Black", 30),
                ("Red", 31),
                ("Green", 32),
                ("Yellow", 33),
                ("Blue", 34),
                ("Magenta", 35),
                ("Cyan", 36),
                ("White", 37),
                ("Bright Black", 90),
                ("Bright Red", 91),
                ("Bright Green", 92),
                ("Bright Yellow", 93),
                ("Bright Blue", 94),
                ("Bright Magenta", 95),
                ("Bright Cyan", 96),
                ("Bright White", 97),
            ];

            for (name, code) in colors {
                let ansi_sequence = format!("\x1b[{}m{}\x1b[0m", code, name);
                let rich_text = ansi_to_rich_text(&ansi_sequence);
                for text in rich_text {
                    ui.label(text);
                }
                ui.label(" ");
            }
        });
    }

    fn show_8bit_example(&self, ui: &mut egui::Ui) {
        ui.heading("8-bit Color (256 Colors) Example");

        ui.horizontal(|ui| {
            ui.label("ANSI Sequence:");
            ui.monospace(display_ansi_sequence(&self.input_8bit));
        });

        ui.label("Conversion Result:");
        let rich_text = ansi_to_rich_text(&self.input_8bit);
        ui.horizontal_wrapped(|ui| {
            for text in rich_text {
                ui.label(text);
            }
        });

        ui.add_space(10.0);

        // Show a portion of the 256-color palette
        ui.label("Partial 256-Color Palette:");
        ui.horizontal_wrapped(|ui| {
            // Show some representative colors
            let sample_colors = [
                (0, "System Black"),
                (1, "System Red"),
                (2, "System Green"),
                (3, "System Yellow"),
                (4, "System Blue"),
                (5, "Magenta"),
                (6, "Cyan"),
                (7, "System White"),
                (16, "RGB Black"),
                (21, "Dark Blue"),
                (46, "Bright Green"),
                (51, "Sky Blue"),
                (196, "Bright Red"),
                (208, "Orange"),
                (220, "Bright Yellow"),
                (232, "Grayscale Black"),
                (240, "Medium Gray"),
                (248, "Light Gray"),
                (255, "Grayscale White"),
            ];

            for (code, name) in sample_colors {
                let ansi_sequence = format!("\x1b[38;5;{}m{}\x1b[0m", code, name);
                let rich_text = ansi_to_rich_text(&ansi_sequence);
                for text in rich_text {
                    ui.label(text);
                }
                ui.label(" ");
            }
        });
    }

    fn show_24bit_example(&self, ui: &mut egui::Ui) {
        ui.heading("24-bit True Color Example");

        ui.horizontal(|ui| {
            ui.label("ANSI Sequence:");
            ui.monospace(display_ansi_sequence(&self.input_24bit));
        });

        ui.label("Conversion Result:");
        let rich_text = ansi_to_rich_text(&self.input_24bit);
        ui.horizontal_wrapped(|ui| {
            for text in rich_text {
                ui.label(text);
            }
        });

        ui.add_space(10.0);

        // Show some color gradients
        ui.label("Color Gradients:");
        ui.horizontal_wrapped(|ui| {
            // Red gradient
            for r in [0, 64, 128, 192, 255] {
                let ansi_sequence = format!("\x1b[38;2;{};0;0mR={}\x1b[0m", r, r);
                let rich_text = ansi_to_rich_text(&ansi_sequence);
                for text in rich_text {
                    ui.label(text);
                }
                ui.label(" ");
            }

            // Green gradient
            for g in [0, 64, 128, 192, 255] {
                let ansi_sequence = format!("\x1b[38;2;0;{};0mG={}\x1b[0m", g, g);
                let rich_text = ansi_to_rich_text(&ansi_sequence);
                for text in rich_text {
                    ui.label(text);
                }
                ui.label(" ");
            }

            // Blue gradient
            for b in [0, 64, 128, 192, 255] {
                let ansi_sequence = format!("\x1b[38;2;0;0;{}mB={}\x1b[0m", b, b);
                let rich_text = ansi_to_rich_text(&ansi_sequence);
                for text in rich_text {
                    ui.label(text);
                }
                ui.label(" ");
            }
        });
    }

    fn show_mixed_example(&self, ui: &mut egui::Ui) {
        ui.heading("Mixed Color Example");

        ui.horizontal(|ui| {
            ui.label("ANSI Sequence:");
            ui.monospace(display_ansi_sequence(&self.input_mixed));
        });

        ui.label("Conversion Result:");
        let rich_text = ansi_to_rich_text(&self.input_mixed);
        ui.horizontal_wrapped(|ui| {
            for text in rich_text {
                ui.label(text);
            }
        });

        ui.add_space(10.0);

        // Segmented display
        ui.label("Segmented Display:");
        let mut parser = AnsiParser::new();
        let colored_segments = parser.parse(&self.input_mixed);

        egui::Grid::new("mixed_grid").num_columns(2).show(ui, |ui| {
            ui.label("Original Text:");
            ui.label("Color");
            ui.end_row();

            for segment in colored_segments {
                ui.label(&segment.text);

                let mut color_desc = "Default".to_string();
                if let Some(fg) = segment.foreground_color {
                    color_desc = format!("Foreground: {:02X}{:02X}{:02X}", fg.r(), fg.g(), fg.b());
                }
                if let Some(bg) = segment.background_color {
                    if !color_desc.is_empty() && color_desc != "Default" {
                        color_desc.push_str(", ");
                    }
                    color_desc.push_str(&format!(
                        "Background: {:02X}{:02X}{:02X}",
                        bg.r(),
                        bg.g(),
                        bg.b()
                    ));
                }

                ui.label(color_desc);
                ui.end_row();
            }
        });
    }

    fn show_custom_input(&mut self, ui: &mut egui::Ui) {
        ui.heading("Custom Input");

        ui.label("Tip: You can type \\x1b as a shortcut for the control character");

        ui.horizontal(|ui| {
            ui.label("ANSI Sequence:");
            ui.text_edit_multiline(&mut self.custom_input);
        });

        // Add some quick buttons for common ANSI sequences
        ui.horizontal(|ui| {
            ui.label("Quick insert:");
            if ui.button("\\x1b[31mRed\\x1b[0m").clicked() {
                self.custom_input = "\\x1b[31mRed\\x1b[0m".to_string();
            }
            if ui.button("\\x1b[38;5;208mOrange\\x1b[0m").clicked() {
                self.custom_input = "\\x1b[38;5;208mOrange\\x1b[0m".to_string();
            }
            if ui.button("\\x1b[38;2;255;105;180mPink\\x1b[0m").clicked() {
                self.custom_input = "\\x1b[38;2;255;105;180mPink\\x1b[0m".to_string();
            }
        });

        ui.label("Readable:");
        ui.monospace(display_ansi_sequence(&self.custom_input));

        ui.label("Conversion Result:");
        // Parse the input to convert \x1b strings to actual control characters
        let parsed_input = parse_ansi_input(&self.custom_input);
        let rich_text = ansi_to_rich_text(&parsed_input);
        ui.horizontal_wrapped(|ui| {
            for text in rich_text {
                ui.label(text);
            }
        });

        ui.add_space(10.0);

        // Show color information
        let mut parser = AnsiParser::new();
        let colored_segments = parser.parse(&parsed_input);

        if !colored_segments.is_empty() {
            ui.collapsing("Color Information", |ui| {
                egui::Grid::new("custom_grid")
                    .num_columns(2)
                    .show(ui, |ui| {
                        ui.label("Text:");
                        ui.label("Color");
                        ui.end_row();

                        for segment in colored_segments {
                            ui.label(&segment.text);

                            let mut color_desc = "Default".to_string();
                            if let Some(fg) = segment.foreground_color {
                                color_desc =
                                    format!("Foreground: RGB({},{},{})", fg.r(), fg.g(), fg.b());
                            }
                            if let Some(bg) = segment.background_color {
                                if !color_desc.is_empty() && color_desc != "Default" {
                                    color_desc.push_str(", ");
                                }
                                color_desc.push_str(&format!(
                                    "Background: RGB({},{},{})",
                                    bg.r(),
                                    bg.g(),
                                    bg.b()
                                ));
                            }

                            ui.label(color_desc);
                            ui.end_row();
                        }
                    });
            });
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "egui_sgr Demo",
        options,
        Box::new(|_cc| Ok(Box::new(AnsiColorDemo::default()))),
    )
}
