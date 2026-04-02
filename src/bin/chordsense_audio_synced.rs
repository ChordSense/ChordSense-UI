use eframe::egui;
use egui_extras::install_image_loaders;
use rodio::{Decoder, DeviceSinkBuilder, Player, Source};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::time::Duration;

const AUDIO_FILE_PATH: &str = "when_i_was_your_man.mp3";
const LAB_FILE_PATH: &str = "when_i_was_your_man.lab";

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

#[derive(Debug, Clone)]
struct ChordSegment {
    start: f64,
    end: f64,
    chord: String,
}

#[derive(Debug, Clone)]
struct LabData {
    chords: Vec<ChordSegment>,
    duration: f64,
}

struct AudioPlayback {
    sink: rodio::stream::MixerDeviceSink,
    player: Player,
    audio_path: PathBuf,
    duration_secs: Option<f64>,
    last_error: Option<String>,
}

impl AudioPlayback {
    fn new<P: Into<PathBuf>>(audio_path: P) -> Self {
        let audio_path = audio_path.into();
        let sink = DeviceSinkBuilder::open_default_sink().expect("open default audio stream");
        let player = Player::connect_new(&sink.mixer());

        let mut this = Self {
            sink,
            player,
            audio_path,
            duration_secs: None,
            last_error: None,
        };

        if let Err(err) = this.load_current_file() {
            this.last_error = Some(err);
        }

        this
    }

    fn load_current_file(&mut self) -> Result<(), String> {
        let player = Player::connect_new(&self.sink.mixer());
        let file = File::open(&self.audio_path)
            .map_err(|e| format!("Could not open audio file '{}': {e}", self.audio_path.display()))?;

        let decoder = Decoder::try_from(file)
            .map_err(|e| format!("Could not decode audio file '{}': {e}", self.audio_path.display()))?;

        self.duration_secs = decoder.total_duration().map(|d| d.as_secs_f64());
        player.append(decoder);
        player.pause();

        self.player = player;
        self.last_error = None;
        Ok(())
    }

    fn play(&self) {
        self.player.play();
    }

    fn pause(&self) {
        self.player.pause();
    }

    fn stop(&mut self) {
        self.player.stop();
        if let Err(err) = self.load_current_file() {
            self.last_error = Some(err);
        }
    }

    fn seek(&mut self, position_secs: f64) {
        let pos = Duration::from_secs_f64(position_secs.max(0.0));
        if let Err(err) = self.player.try_seek(pos) {
            self.last_error = Some(format!("Seek failed: {err}"));
        }
    }

    fn position_secs(&self) -> f64 {
        self.player.get_pos().as_secs_f64()
    }

    fn is_paused(&self) -> bool {
        self.player.is_paused()
    }

    fn is_finished(&self) -> bool {
        self.player.empty()
    }

    fn duration_secs(&self) -> Option<f64> {
        self.duration_secs
    }

    fn path_label(&self) -> String {
        self.audio_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("audio file")
            .to_string()
    }
}

struct MyEguiApp {
    started: bool,
    mode: usize,

    progress: f64,
    is_playing: bool,

    chord_data: Option<LabData>,
    chord_assets: HashMap<&'static str, &'static str>,
    audio: AudioPlayback,
}

impl Default for MyEguiApp {
    fn default() -> Self {
        Self {
            started: false,
            mode: 0,
            progress: 0.0,
            is_playing: false,
            chord_data: load_lab_file(LAB_FILE_PATH),
            chord_assets: chord_asset_map(),
            audio: AudioPlayback::new(AUDIO_FILE_PATH),
        }
    }
}

impl MyEguiApp {
    fn max_duration(&self) -> f64 {
        let lab_duration = self.chord_data.as_ref().map(|d| d.duration).unwrap_or(0.0);
        let audio_duration = self.audio.duration_secs().unwrap_or(0.0);
        lab_duration.max(audio_duration)
    }

    fn sync_progress_from_audio(&mut self) {
        self.progress = self.audio.position_secs().min(self.max_duration());

        if self.audio.is_finished() {
            self.is_playing = false;
        } else {
            self.is_playing = !self.audio.is_paused();
        }
    }

    fn play(&mut self) {
        let max_duration = self.max_duration();
        if max_duration > 0.0 && self.progress >= max_duration {
            self.stop();
        }

        self.audio.play();
        self.sync_progress_from_audio();
    }

    fn pause(&mut self) {
        self.audio.pause();
        self.sync_progress_from_audio();
    }

    fn stop(&mut self) {
        self.audio.stop();
        self.sync_progress_from_audio();
    }

    fn seek(&mut self, new_position: f64) {
        let target = new_position.clamp(0.0, self.max_duration());
        self.audio.seek(target);
        self.sync_progress_from_audio();
    }
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.started {
            let any_pressed = ctx.input(|i| !i.keys_down.is_empty());
            if any_pressed {
                self.started = true;
            }
        }

        ctx.set_visuals(egui::Visuals::light());

        if self.started && ctx.input(|i| i.key_pressed(egui::Key::M)) {
            self.pause();
            self.mode = (self.mode + 1) % 2;
        }

        self.sync_progress_from_audio();

        if self.started && self.mode == 0 && self.is_playing {
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
                0 => show_sense_mode(ui, ctx, self),
                1 => show_record_mode(ui),
                _ => {}
            }
        });
    }
}

fn load_lab_file(path: &str) -> Option<LabData> {
    let text = fs::read_to_string(path).ok()?;
    let mut chords = Vec::new();
    let mut duration = 0.0_f64;

    for (line_no, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 {
            eprintln!("Skipping malformed .lab line {}: {}", line_no + 1, line);
            continue;
        }

        let start = match parts[0].parse::<f64>() {
            Ok(v) => v,
            Err(_) => {
                eprintln!("Bad start time on .lab line {}: {}", line_no + 1, line);
                continue;
            }
        };

        let end = match parts[1].parse::<f64>() {
            Ok(v) => v,
            Err(_) => {
                eprintln!("Bad end time on .lab line {}: {}", line_no + 1, line);
                continue;
            }
        };

        let chord = parts[2..].join(" ");
        duration = duration.max(end);
        chords.push(ChordSegment { start, end, chord });
    }

    if chords.is_empty() {
        None
    } else {
        Some(LabData { chords, duration })
    }
}

fn active_chord<'a>(time: f64, chords: &'a [ChordSegment]) -> Option<&'a ChordSegment> {
    chords.iter().find(|seg| time >= seg.start && time < seg.end)
}

fn next_chord<'a>(time: f64, chords: &'a [ChordSegment]) -> Option<&'a ChordSegment> {
    chords.iter().find(|seg| seg.start > time)
}

fn previous_chord<'a>(time: f64, chords: &'a [ChordSegment]) -> Option<&'a ChordSegment> {
    chords.iter().rev().find(|seg| seg.end <= time)
}

fn pretty_chord_label(raw: &str) -> String {
    if raw == "N" {
        return "No chord".to_string();
    }

    let mut label = raw.replace(":maj", "");
    label = label.replace(":min", "m");

    if let Some((base, bass)) = label.split_once('/') {
        let bass_note = bass_degree_to_note(raw, bass).unwrap_or_else(|| bass.to_string());
        format!("{}/{}", base, bass_note)
    } else {
        label
    }
}

fn bass_degree_to_note(raw: &str, bass: &str) -> Option<String> {
    let root = chord_root(raw)?;
    let note = match bass {
        "1" => root,
        "b2" => transpose_note(root, 1)?,
        "2" => transpose_note(root, 2)?,
        "b3" => transpose_note(root, 3)?,
        "3" => transpose_note(root, 4)?,
        "4" => transpose_note(root, 5)?,
        "b5" => transpose_note(root, 6)?,
        "5" => transpose_note(root, 7)?,
        "#5" | "b6" => transpose_note(root, 8)?,
        "6" => transpose_note(root, 9)?,
        "b7" => transpose_note(root, 10)?,
        "7" => transpose_note(root, 11)?,
        _ => return None,
    };

    Some(note.to_string())
}

fn chord_root(raw: &str) -> Option<&str> {
    let root_part = raw.split(':').next()?;
    if root_part == "N" {
        None
    } else {
        Some(root_part)
    }
}

fn transpose_note(root: &str, semitones: usize) -> Option<&'static str> {
    let chromatic = [
        "C", "C#", "D", "Eb", "E", "F", "F#", "G", "Ab", "A", "Bb", "B",
    ];

    let idx = match root {
        "C" => 0,
        "C#" | "Db" => 1,
        "D" => 2,
        "D#" | "Eb" => 3,
        "E" => 4,
        "F" => 5,
        "F#" | "Gb" => 6,
        "G" => 7,
        "G#" | "Ab" => 8,
        "A" => 9,
        "A#" | "Bb" => 10,
        "B" | "Cb" => 11,
        _ => return None,
    };

    Some(chromatic[(idx + semitones) % 12])
}

fn chord_asset_key(raw: &str) -> Option<&'static str> {
    let simplified = simplify_for_asset(raw);

    match simplified.as_str() {
        "A" => Some("a"),
        "Ab" | "G#" => Some("ab"),
        "Abm" | "G#m" => Some("abm"),
        "Am" => Some("am"),
        "B" => Some("b"),
        "Bb" | "A#" => Some("bb"),
        "Bbm" | "A#m" => Some("bbm"),
        "Bm" => Some("bm"),
        "C" => Some("c"),
        "C#" | "Db" => Some("c#"),
        "C#m" | "Dbm" => Some("c#m"),
        "Cm" => Some("cm"),
        "D" => Some("d"),
        "Dm" => Some("dm"),
        "E" => Some("e"),
        "Eb" | "D#" => Some("eb"),
        "Ebm" | "D#m" => Some("ebm"),
        "Em" => Some("em"),
        "F" => Some("f"),
        "F#m" | "Gbm" => Some("f#m"),
        "Fm" => Some("fm"),
        "G" => Some("g"),
        "Gm" => Some("gm"),
        _ => None,
    }
}

fn simplify_for_asset(raw: &str) -> String {
    if raw == "N" {
        return "N".to_string();
    }

    let base = raw.split('/').next().unwrap_or(raw);
    let root = base.split(':').next().unwrap_or(base);
    let quality = base.split(':').nth(1).unwrap_or("maj").to_lowercase();

    let is_minor_family = quality.starts_with("min");

    if is_minor_family {
        format!("{}m", root)
    } else {
        root.to_string()
    }
}

fn chord_asset_map() -> HashMap<&'static str, &'static str> {
    HashMap::from([
        ("a", "assets/chords/a.png"),
        ("ab", "assets/chords/ab.png"),
        ("abm", "assets/chords/abm.png"),
        ("am", "assets/chords/am.png"),
        ("b", "assets/chords/b.png"),
        ("bb", "assets/chords/bb.png"),
        ("bbm", "assets/chords/bbm.png"),
        ("bm", "assets/chords/bm.png"),
        ("c", "assets/chords/c.png"),
        ("c#", "assets/chords/c#.png"),
        ("c#m", "assets/chords/c#m.png"),
        ("cm", "assets/chords/cm.png"),
        ("d", "assets/chords/d.png"),
        ("dm", "assets/chords/dm.png"),
        ("e", "assets/chords/e.png"),
        ("eb", "assets/chords/eb.png"),
        ("ebm", "assets/chords/ebm.png"),
        ("em", "assets/chords/em.png"),
        ("f", "assets/chords/f.png"),
        ("f#m", "assets/chords/f#m.png"),
        ("fm", "assets/chords/fm.png"),
        ("g", "assets/chords/g.png"),
        ("gm", "assets/chords/gm.png"),
    ])
}

fn show_start_screen(ui: &mut egui::Ui) {
    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
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

fn show_sense_mode(ui: &mut egui::Ui, ctx: &egui::Context, app: &mut MyEguiApp) {
    let max_duration = app.max_duration();

    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
        ui.add_space(20.0);
        ui.heading(egui::RichText::new("Mode: Play Along").size(48.0));
        ui.add_space(20.0);

        let back = egui::Image::new(egui::include_image!("../../assets/icons/back.png"))
            .fit_to_exact_size(egui::vec2(50.0, 50.0));
        let pause = egui::Image::new(egui::include_image!("../../assets/icons/pause.png"))
            .fit_to_exact_size(egui::vec2(50.0, 50.0));
        let play_button = egui::Image::new(egui::include_image!("../../assets/icons/play-button.png"))
            .fit_to_exact_size(egui::vec2(50.0, 50.0));

        ui.horizontal(|ui| {
            ui.add_space(20.0);
            let metronome = egui::Image::new(egui::include_image!("../../assets/icons/metronome.png"))
                .fit_to_exact_size(egui::vec2(50.0, 50.0));
            ui.add(metronome);
            ui.add_space(9.0);

            let back_response = ui.add(back.sense(egui::Sense::click()));
            if back_response.clicked() {
                app.stop();
            }

            ui.add_space(12.0);

            if app.is_playing {
                let pause_response = ui.add(pause.sense(egui::Sense::click()));
                if pause_response.clicked() {
                    app.pause();
                }
            } else {
                let play_response = ui.add(play_button.sense(egui::Sense::click()));
                if play_response.clicked() {
                    app.play();
                    ctx.request_repaint();
                }
            }

            ui.add_space(12.0);

            let mut slider_value = app.progress;
            let slider_response = ui.add_sized(
                [500.0, 30.0],
                egui::Slider::new(&mut slider_value, 0.0..=max_duration)
                    .show_value(false)
                    .min_decimals(0)
                    .max_decimals(3),
            );

            if slider_response.changed() {
                let was_playing = app.is_playing;
                app.seek(slider_value);
                if was_playing {
                    app.play();
                }
            }

            ui.add_space(12.0);
            ui.label(
                egui::RichText::new(format!("{:.3}s / {:.3}s", app.progress, max_duration))
                    .size(22.0),
            );
        });

        let now_pos = app.progress;

        let (
            active_label,
            previous_label,
            next_label,
            active_image_path,
            active_model_output,
            active_simplified_output,
        ) = if let Some(data) = &app.chord_data {
            let active = active_chord(now_pos, &data.chords);
            let previous = previous_chord(now_pos, &data.chords);
            let next = next_chord(now_pos, &data.chords);

            let active_label = active
                .map(|c| pretty_chord_label(&c.chord))
                .unwrap_or_else(|| "No chord".to_string());

            let previous_label = previous
                .map(|c| pretty_chord_label(&c.chord))
                .unwrap_or_else(|| "--".to_string());

            let next_label = next
                .map(|c| pretty_chord_label(&c.chord))
                .unwrap_or_else(|| "--".to_string());

            let active_image_path = active
                .and_then(|c| chord_asset_key(&c.chord))
                .and_then(|key| app.chord_assets.get(key).copied())
                .map(|s| s.to_string());

            let active_model_output = active.map(|c| c.chord.clone());
            let active_simplified_output = active.map(|c| simplify_for_asset(&c.chord));

            (
                active_label,
                previous_label,
                next_label,
                active_image_path,
                active_model_output,
                active_simplified_output,
            )
        } else {
            (
                "No chord".to_string(),
                "--".to_string(),
                "--".to_string(),
                None,
                None,
                None,
            )
        };

        ui.add_space(10.0);
        ui.label(egui::RichText::new(format!("Current Chord: {}", active_label)).size(34.0));

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(format!("Previous: {}", previous_label)).size(22.0));
            ui.add_space(20.0);
            ui.label(egui::RichText::new(format!("Next: {}", next_label)).size(22.0));
        });

        ui.add_space(16.0);

        if let Some(path) = active_image_path {
            let chord_image = egui::Image::new(format!("file://{}", path))
                .fit_to_exact_size(egui::vec2(450.0, 530.0));
            ui.add(chord_image);
        } else {
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.set_min_size(egui::vec2(450.0, 200.0));
                ui.vertical_centered(|ui| {
                    ui.add_space(40.0);
                    ui.label(egui::RichText::new("No matching chord diagram asset").size(28.0));
                    if let Some(model_output) = &active_model_output {
                        ui.label(
                            egui::RichText::new(format!("Model output: {}", model_output)).size(22.0),
                        );
                    }
                    if let Some(simplified_output) = &active_simplified_output {
                        ui.label(
                            egui::RichText::new(format!("Simplified for asset: {}", simplified_output))
                                .size(20.0),
                        );
                    }
                });
            });
        }

        ui.add_space(12.0);

        ui.separator();
        ui.label(egui::RichText::new("Audio Debug").size(24.0));
        ui.label(
            egui::RichText::new(format!("Audio file: {}", app.audio.path_label())).size(18.0),
        );
        if let Some(duration) = app.audio.duration_secs() {
            ui.label(
                egui::RichText::new(format!("Decoded audio duration: {:.6}s", duration)).size(18.0),
            );
        } else {
            ui.label(egui::RichText::new("Decoded audio duration: unavailable").size(18.0));
        }
        ui.label(
            egui::RichText::new(format!("Playback position from audio engine: {:.6}s", app.progress))
                .size(18.0),
        );
        ui.label(
            egui::RichText::new(format!("Audio paused: {}", app.audio.is_paused())).size(18.0),
        );
        ui.label(
            egui::RichText::new(format!("Audio queue empty: {}", app.audio.is_finished())).size(18.0),
        );
        if let Some(err) = &app.audio.last_error {
            ui.label(
                egui::RichText::new(format!("Audio status: {}", err))
                    .size(18.0)
                    .color(egui::Color32::RED),
            );
        } else {
            ui.label(
                egui::RichText::new("Audio status: loaded").size(18.0).color(egui::Color32::DARK_GREEN),
            );
        }

        if let Some(data) = &app.chord_data {
            ui.separator();
            ui.label(egui::RichText::new("Detected chord timeline").size(26.0));
            ui.label(
                egui::RichText::new(format!(
                    "Playback duration is based on max(audio duration, .lab end time): {:.3}s",
                    max_duration
                ))
                .size(18.0)
                .color(egui::Color32::DARK_GRAY),
            );
            ui.label(
                egui::RichText::new(format!(".lab end time: {:.3}s", data.duration))
                    .size(18.0)
                    .color(egui::Color32::DARK_GRAY),
            );

            egui::ScrollArea::vertical().max_height(180.0).show(ui, |ui| {
                for seg in &data.chords {
                    let is_active = app.progress >= seg.start && app.progress < seg.end;
                    let text = format!(
                        "{:.3} - {:.3}    {}",
                        seg.start,
                        seg.end,
                        pretty_chord_label(&seg.chord)
                    );

                    if is_active {
                        ui.label(
                            egui::RichText::new(text)
                                .size(22.0)
                                .strong()
                                .color(egui::Color32::from_rgb(0, 90, 200)),
                        );
                    } else {
                        ui.label(egui::RichText::new(text).size(20.0));
                    }
                }
            });
        } else {
            ui.label(egui::RichText::new("Could not load song.lab").size(24.0));
        }

        ui.add_space(8.0);
        ui.label(egui::RichText::new("Press M to switch modes").size(25.0));
    });
}

fn show_record_mode(ui: &mut egui::Ui) {
    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
        ui.add_space(20.0);
        ui.heading(egui::RichText::new("Mode: RECORD").size(90.0));

        let rec_image = egui::Image::new(egui::include_image!("../../assets/icons/record_circle.png"))
            .fit_to_exact_size(egui::vec2(50.0, 50.0));
        ui.add(rec_image);

        ui.add_space(2.0);
        ui.label(egui::RichText::new("Press M to switch modes").size(25.0));
    });
}
