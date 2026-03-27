// use eframe::egui;

// fn main() -> eframe::Result<()> {
//     let native_options = eframe::NativeOptions::default();

//     eframe::run_native(
//         "My egui App",
//         native_options,
//         Box::new(|_cc| Ok(Box::new(MyEguiApp::default()))),
//     )
// }

// #[derive(Default)]
// struct MyEguiApp {}

// impl eframe::App for MyEguiApp {
//     fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
//         egui::CentralPanel::default().show(ctx, |ui| {
//             ui.heading("Hello World!");
//             ui.label("ChordSense says hello!");
//         });
//     }
// }

use eframe::egui;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_fullscreen(true),
        ..Default::default()
    };


    eframe::run_native(
        "ChordSense",
        native_options,
        Box::new(|_cc| Ok(Box::new(MyEguiApp::default()))),
    )
}

struct MyEguiApp {
    started: bool,
    mode: usize,
}

// Set up default values
impl Default for MyEguiApp {
    fn default() -> Self {
        Self { 
            started: false,
            mode: 0, 
        }
    }
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Start on any key or mouse click
        if !self.started {
            let any_pressed = ctx.input(|i| {
                !i.keys_down.is_empty()});
            if any_pressed {
                self.started = true;
            }
        }

        ctx.set_visuals(egui::Visuals::light());

        if self.started && ctx.input(|i| i.key_pressed(egui::Key::M)) {
            self.mode = (self.mode + 1) % 3;
        }

        // Footer stays anchored at the bottom
        egui::TopBottomPanel::bottom("footer")
            .exact_height(55.0)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(
                        egui::RichText::new("MVP Demo • HDMI Display • Rust + egui")
                            .size(24.0)
                            .color(egui::Color32::GRAY),
                    );
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            if !self.started {
                show_start_screen(ui);
                return;
            }

            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.add_space((ui.available_height() * 0.18).clamp(20.0, 120.0));

                match self.mode {
                    0 => {
                        ui.heading(egui::RichText::new("SENSE MODE").size(90.0));
                        ui.label(egui::RichText::new("G Major").size(65.0));
                    }
                    1 => {
                        ui.heading(egui::RichText::new("RT-SENSE MODE").size(90.0));
                        ui.label(egui::RichText::new("A3").size(65.0));
                    }
                    _ => {
                        ui.heading(egui::RichText::new("RECORD MODE").size(90.0));
                    }
                }

                ui.add_space(40.0);
                ui.label(egui::RichText::new("Press M to switch modes").size(42.0));
            });
        });
    }
}


fn show_start_screen(ui: &mut egui::Ui) {
    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
        // Push the card down a bit, but scale with screen height
        ui.add_space((ui.available_height() * 0.22).clamp(20.0, 140.0));

        let card_width = ui.available_width().min(900.0);

        ui.allocate_ui_with_layout(
            egui::vec2(card_width, 0.0),
            egui::Layout::top_down(egui::Align::Center),
            |ui| {
                egui::Frame::new()
                    .fill(egui::Color32::from_gray(70))
                    .stroke(egui::Stroke::new(2.0, egui::Color32::WHITE))
                    .corner_radius(egui::CornerRadius::same(20))
                    .inner_margin(egui::Margin::same(30))
                    .show(ui, |ui| {
                        ui.horizontal_centered(|ui| {
                            ui.add_space(180.0);
                            ui.label(
                                egui::RichText::new("♪")
                                    .size(48.0)
                                    .color(egui::Color32::from_rgb(80, 160, 255)),
                            );
                            ui.label(
                                egui::RichText::new("♫")
                                    .size(48.0)
                                    .color(egui::Color32::from_rgb(242, 155, 47)),
                            );
                             ui.add_space(15.0);
                            ui.label(
                                egui::RichText::new("ChordSense")
                                    .size(72.0)
                                    .color(egui::Color32::WHITE),
                            );

                        });

                        ui.add_space(24.0);

                        ui.label(
                            egui::RichText::new("Press any button to start")
                                .size(38.0)
                                .color(egui::Color32::WHITE),
                        );
                    });
            },
        );
    });
}