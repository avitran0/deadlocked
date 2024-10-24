use eframe::egui::{self, Ui};
use std::{collections::HashMap, sync::mpsc};
use strum::IntoEnumIterator;

use crate::{
    colors::Colors,
    config::{parse_config, AimbotConfig, AimbotStatus, CONFIG_FILE_NAME},
    key_codes::KeyCode,
    message::{Game, Message},
};

pub struct Gui {
    tx: Vec<mpsc::Sender<Message>>,
    rx: mpsc::Receiver<Message>,
    config: HashMap<Game, AimbotConfig>,
    status: HashMap<Game, AimbotStatus>,
    current_game: Game,
}

impl Gui {
    pub fn new(tx: Vec<mpsc::Sender<Message>>, rx: mpsc::Receiver<Message>) -> Self {
        // read config
        let mut config = parse_config();
        config.get_mut(&Game::CS2).unwrap().enabled = true;
        let mut status = HashMap::new();
        for game in Game::iter() {
            status.insert(game, AimbotStatus::GameNotStarted);
        }
        let out = Self {
            tx,
            rx,
            config,
            status,
            current_game: Game::CS2,
        };
        out.send_message(Message::ConfigEnabled(Game::CS2, true));
        out.write_config();
        out
    }

    fn send_message(&self, message: Message) {
        for tx in &self.tx {
            if let Err(error) = tx.send(message) {
                println!("{}", error);
            }
        }
        self.write_config();
    }

    fn add_game_grid(&mut self, ui: &mut Ui, game: Game) {
        egui::Grid::new(game.string())
            .num_columns(2)
            .min_col_width(75.0)
            .show(ui, |ui| {
                ui.label("Enabled");
                if ui
                    .checkbox(&mut self.config.get_mut(&game).unwrap().enabled, "")
                    .changed()
                {
                    self.send_message(Message::ConfigEnabled(
                        game,
                        self.config.get(&game).unwrap().enabled,
                    ));
                }
                ui.end_row();

                ui.label("Hotkey")
                    .on_hover_text("which key or mouse button should activate the aimbot");
                egui::ComboBox::new(format!("{}_hotkey", game.string()), "")
                    .selected_text(format!("{:?}", self.config.get(&game).unwrap().hotkey))
                    .show_ui(ui, |ui| {
                        for key_code in KeyCode::iter() {
                            let text = format!("{:?}", &key_code);
                            if ui
                                .selectable_value(
                                    &mut self.config.get_mut(&game).unwrap().hotkey,
                                    key_code,
                                    text,
                                )
                                .clicked()
                            {
                                self.send_message(Message::ConfigHotkey(
                                    game,
                                    self.config.get(&game).unwrap().hotkey,
                                ));
                            }
                        }
                    });
                ui.end_row();

                ui.label("Start Bullet")
                    .on_hover_text("after how many bullets fired in a row the aimbot should start");
                if ui
                    .add(egui::Slider::new(
                        &mut self.config.get_mut(&game).unwrap().start_bullet,
                        0..=5,
                    ))
                    .changed()
                {
                    self.send_message(Message::ConfigStartBullet(
                        game,
                        self.config.get(&game).unwrap().start_bullet,
                    ));
                }
                ui.end_row();

                ui.label("Aim Lock")
                    .on_hover_text("whether the aim should lock onto the target");
                if ui
                    .checkbox(&mut self.config.get_mut(&game).unwrap().aim_lock, "")
                    .changed()
                {
                    self.send_message(Message::ConfigAimLock(
                        game,
                        self.config.get(&game).unwrap().aim_lock,
                    ));
                }
                ui.end_row();

                ui.label("Visibility Check")
                    .on_hover_text("whether to check for player visibility");
                if ui
                    .checkbox(
                        &mut self.config.get_mut(&game).unwrap().visibility_check,
                        "",
                    )
                    .changed()
                {
                    self.send_message(Message::ConfigVisibilityCheck(
                        game,
                        self.config.get(&game).unwrap().visibility_check,
                    ));
                }
                ui.end_row();

                ui.label("FOV")
                    .on_hover_text("how much around the crosshair the aimbot should \"see\"");
                if ui
                    .add(
                        egui::Slider::new(&mut self.config.get_mut(&game).unwrap().fov, 0.1..=10.0)
                            .suffix("°"),
                    )
                    .changed()
                {
                    self.send_message(Message::ConfigFOV(
                        game,
                        self.config.get(&game).unwrap().fov,
                    ));
                }
                ui.end_row();

                ui.label("Smooth")
                    .on_hover_text("how much the aimbot input should be smoothed, higher is more");
                if ui
                    .add(egui::Slider::new(
                        &mut self.config.get_mut(&game).unwrap().smooth,
                        1.0..=10.0,
                    ))
                    .changed()
                {
                    self.send_message(Message::ConfigSmooth(
                        game,
                        self.config.get(&game).unwrap().smooth,
                    ));
                }
                ui.end_row();

                ui.label("Multibone").on_hover_text(
                    "whether the aimbot should aim at all of the body, or just the head",
                );
                if ui
                    .checkbox(&mut self.config.get_mut(&game).unwrap().multibone, "")
                    .changed()
                {
                    self.send_message(Message::ConfigMultibone(
                        game,
                        self.config.get(&game).unwrap().multibone,
                    ));
                }
                ui.end_row();
            });
    }

    fn add_game_status(&mut self, ui: &mut Ui, game: Game) {
        let status = self.status.get(&game).unwrap();
        ui.label(
            egui::RichText::new(status.string())
                .heading()
                .color(match status {
                    AimbotStatus::Working => Colors::GREEN,
                    AimbotStatus::GameNotStarted => Colors::YELLOW,
                    AimbotStatus::Paused => Colors::YELLOW,
                }),
        );
    }

    fn write_config(&self) {
        let out = toml::to_string(&self.config).unwrap();
        std::fs::write(CONFIG_FILE_NAME, out).unwrap();
    }
}

impl eframe::App for Gui {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(Message::Status(game, status)) = self.rx.try_recv() {
            *self.status.get_mut(&game).unwrap() = status
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let previous_game = self.current_game;
            egui::ComboBox::new("game", "Current Game")
                .selected_text(self.current_game.upper_string())
                .show_ui(ui, |ui| {
                    for game in Game::iter() {
                        let text = game.upper_string();
                        if ui
                            .selectable_value(&mut self.current_game, game, text)
                            .clicked()
                        {
                            self.send_message(Message::ConfigEnabled(previous_game, false));
                            self.send_message(Message::ConfigEnabled(self.current_game, true));

                            self.config.get_mut(&previous_game).unwrap().enabled = false;
                            self.config.get_mut(&self.current_game).unwrap().enabled = true;

                            self.write_config();
                        }
                    }
                });
            ui.separator();
            egui::Grid::new("main_grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .show(ui, |ui| {
                    self.add_game_grid(ui, self.current_game);
                    self.add_game_status(ui, self.current_game);
                });
        });
    }
}
