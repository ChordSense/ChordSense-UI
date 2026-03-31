use eframe::egui;
use egui_extras::install_image_loaders;
use std::time::{Duration, Instant};

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_fullscreen(true),
        ..Default::default()
    };


    eframe::run_native(
        "ChordSense",
        native_options,
        Box::new(|cc| {
        install_image_loaders(&cc.egui_ctx);
        Ok(Box::new(MyEguiApp::default()))
        }),
    )
}

struct MyEguiApp {
    started: bool,
    mode: usize,
    progress: f32,
    is_playing: bool,
    last_update: Instant,
    image_names: Vec<&'static str>,
    current_image: usize,
}

// Set up default values
impl Default for MyEguiApp {
    fn default() -> Self {
        Self { 
            started: false,
            mode: 0, 
            progress: 0.0,
            is_playing: false,
            last_update: Instant::now(),
            image_names: vec![
                "assets/chords/a.png",
                "assets/chords/ab.png",
                "assets/chords/abm.png",
                "assets/chords/am.png",
                "assets/chords/b.png",
                "assets/chords/bb.png",
                "assets/chords/bbm.png",
                "assets/chords/bm.png",
                "assets/chords/c.png",
                "assets/chords/c#.png",
                "assets/chords/c#m.png",
                "assets/chords/cm.png",
                "assets/chords/d.png",
                "assets/chords/dm.png",
                "assets/chords/e.png",
                "assets/chords/eb.png",
                "assets/chords/ebm.png",
                "assets/chords/em.png",
                "assets/chords/f.png",
                "assets/chords/f#m.png",
                "assets/chords/fm.png",
                "assets/chords/g.png",
                "assets/chords/gm.png",
            ],
            current_image: 0,
        }
    }
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.started {
            let any_pressed = ctx.input(|i| {
                !i.keys_down.is_empty()
            });
            if any_pressed {
                self.started = true;
            }
        }

        ctx.set_visuals(egui::Visuals::light());

        if self.started && ctx.input(|i| i.key_pressed(egui::Key::M)) {
            self.mode = (self.mode + 1) % 2;
        }

        // Press right arrow to go to next chord image
        if ctx.input(|i| i.key_pressed(egui::Key::ArrowRight)) {
            self.current_image = (self.current_image + 1) % self.image_names.len();
        }

        // Press left arrow to go to previous chord image
        if ctx.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
            if self.current_image == 0 {
                self.current_image = self.image_names.len() - 1;
            } else {
                self.current_image -= 1;
            }
        }


        if self.started && self.mode == 0 && self.is_playing {
            let now = Instant::now();
            let dt = now.duration_since(self.last_update).as_secs_f32();
            self.last_update = now;

            self.progress += dt;
            if self.progress >= 120.0 {
                self.progress = 120.0;
                self.is_playing = false;
            }

            ctx.request_repaint_after(Duration::from_millis(16));
        }

        egui::TopBottomPanel::bottom("footer")
            .exact_height(55.0)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(
                        egui::RichText::new("Demo • HDMI Display • Rust + egui")
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
            
            match self.mode {
                0 => {
                    let chord_path = self.image_names[self.current_image];
                    show_sense_mode(ui, &mut self.progress, &mut self.is_playing, &mut self.last_update, chord_path);
                }
                1 => show_record_mode(ui),
                _ => {}
            }
  
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


fn show_sense_mode( ui: &mut egui::Ui,progress: &mut f32, is_playing: &mut bool, last_update: &mut Instant, chord_path: &str,) {

    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
        ui.add_space(20.0);

        ui.heading(egui::RichText::new("Mode: Play Along").size(48.0));
        ui.add_space(20.0);



        let back = egui::Image::new(egui::include_image!("../assets/icons/back.png"))
            .fit_to_exact_size(egui::vec2(50.0, 50.0));

        let pause = egui::Image::new(egui::include_image!("../assets/icons/pause.png"))
            .fit_to_exact_size(egui::vec2(50.0, 50.0));

        let play_button = egui::Image::new(egui::include_image!("../assets/icons/play-button.png"))
            .fit_to_exact_size(egui::vec2(50.0, 50.0));

        let chord = egui::Image::new(format!("file://{}", chord_path))
            .fit_to_exact_size(egui::vec2(450.0, 530.0));

        

        ui.horizontal(|ui| {
            ui.add_space(20.0);
            let metronome = egui::Image::new(egui::include_image!("../assets/icons/metronome.png"))
                .fit_to_exact_size(egui::vec2(50.0, 50.0));
            ui.add(metronome);
            ui.add_space(9.0);
            let back_response = ui.add(back.sense(egui::Sense::click()));
            if back_response.clicked() {
                *progress = 0.0;
                *is_playing = false;
            }

            ui.add_space(12.0);

            if *is_playing {
                let pause_response = ui.add(pause.sense(egui::Sense::click()));
                if pause_response.clicked() {
                    *is_playing = false;
                }
            } else {
                let play_response = ui.add(play_button.sense(egui::Sense::click()));
                if play_response.clicked() {
                    *is_playing = true;
                    *last_update = Instant::now();
                    ui.ctx().request_repaint(); 
                }
            }

            ui.add_space(12.0);

            ui.add_sized(
                [500.0, 30.0],
                egui::Slider::new(progress, 0.0..=120.0)
                    .show_value(false)
                    .min_decimals(0)
                    .max_decimals(0),
            );
        });

        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.add_space(20.0);
            let vinyl = egui::Image::new(egui::include_image!("../assets/icons/vinyl.png"))
            .fit_to_exact_size(egui::vec2(50.0, 50.0));
            ui.add(vinyl);
            ui.add_space(10.0);
            ui.label(egui::RichText::new("Currently Playing: Aerosmith - Walk This Way").size(30.0));
            
        });


        ui.horizontal(|ui| {
            ui.add_space(20.0);
            let music_note = egui::Image::new(egui::include_image!("../assets/icons/music_note.png"))
                .fit_to_exact_size(egui::vec2(50.0, 50.0));

            
            ui.add(music_note);
            ui.add_space(10.0);
            ui.label(egui::RichText::new("BPM: 122").size(30.0));
            

        });

        ui.add_space(20.0);
        ui.add(chord);

        ui.add_space(2.0);
        ui.label(egui::RichText::new("Press M to switch modes").size(25.0));

        
        
    });

    

    
}

fn show_record_mode(ui: &mut egui::Ui) {

     ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
        ui.add_space(20.0);
        ui.heading(egui::RichText::new("Mode: RECORD").size(90.0));

        let rec_image = egui::Image::new(egui::include_image!("../assets/icons/record_circle.png")).fit_to_exact_size(egui::vec2(50.0, 50.0));
            ui.add(rec_image);

        ui.add_space(2.0);
        ui.label(egui::RichText::new("Press M to switch modes").size(25.0));

     });
}