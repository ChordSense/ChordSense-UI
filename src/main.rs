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
    mode: usize,
}

impl Default for MyEguiApp {
    fn default() -> Self {
        Self { mode: 0 }
    }
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(egui::Visuals::light());
        if ctx.input(|i| i.key_pressed(egui::Key::M)) {
            self.mode = (self.mode + 1) % 3;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(70.0);

                match self.mode {
                    0 => {
                        ui.heading(egui::RichText::new("SENSE MODE").size(100.0));
                        ui.label(egui::RichText::new("G Major").size(70.0));
                        
                    }
                    1 => {
                        ui.heading(egui::RichText::new("RT-SENSE MODE").size(100.0));
                        ui.label(egui::RichText::new("A3").size(70.0));
                    }
                    _ => {
                        ui.heading(egui::RichText::new("RECORD MODE").size(100.0));
                    }
                }

                ui.add_space(50.0);
                ui.label(egui::RichText::new("Press M to switch modes").size(60.0));
            });
        });
    }
}