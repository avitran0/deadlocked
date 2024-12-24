use eframe::egui::{self, vec2, Align, Align2, Color32, Layout, Ui};
use log::info;
use std::sync::mpsc;
use strum::IntoEnumIterator;

use crate::{
    color::Colors,
    config::{parse_config, write_config, AimbotConfig, AimbotStatus, Config, VERSION},
    key_codes::KeyCode,
    message::{Game, Message},
    mouse::MouseStatus,
};

pub struct Gui {
    tx_aimbot: mpsc::Sender<Message>,
    rx: mpsc::Receiver<Message>,
    config: Config,
    status: AimbotStatus,
    mouse_status: MouseStatus,
    frame_times: Vec<f64>,
    average_frame_time: f64,
}

impl Gui {
    pub fn new(tx_aimbot: mpsc::Sender<Message>, rx: mpsc::Receiver<Message>) -> Self {
        // read config
        let config = parse_config();
        let status = AimbotStatus::GameNotStarted;
        let out = Self {
            tx_aimbot,
            rx,
            config,
            status,
            mouse_status: MouseStatus::NoMouseFound,
            frame_times: Vec::with_capacity(50),
            average_frame_time: 0.0,
        };
        write_config(&out.config);
        out
    }

    fn send_message(&self, message: Message) {
        self.tx_aimbot.send(message).expect("aimbot thread died");
    }

    fn aimbot_grid(&mut self, ui: &mut Ui) {
        let mut game_config = self
            .config
            .games
            .get_mut(&self.config.current_game)
            .unwrap()
            .clone();

        egui::Grid::new("aimbot")
            .num_columns(2)
            .min_col_width(80.0)
            .show(ui, |ui| {
                ui.label("Enable Aimbot")
                    .on_hover_text("general aimbot enable");
                if ui.checkbox(&mut game_config.enabled, "").changed() {
                    self.send_message(Message::ConfigEnableAimbot(game_config.enabled));
                    self.write_game_config(&game_config);
                }

                ui.label("Hotkey")
                    .on_hover_text("which key or mouse button should activate the aimbot");
                egui::ComboBox::new("aimbot_hotkey", "")
                    .selected_text(format!("{:?}", game_config.hotkey))
                    .show_ui(ui, |ui| {
                        for key_code in KeyCode::iter() {
                            let text = format!("{:?}", &key_code);
                            if ui
                                .selectable_value(&mut game_config.hotkey, key_code, text)
                                .clicked()
                            {
                                self.send_message(Message::ConfigHotkey(game_config.hotkey));
                                self.write_game_config(&game_config);
                            }
                        }
                    });
                ui.end_row();

                ui.label("Aim Lock")
                    .on_hover_text("whether the aim should lock onto the target");
                if ui.checkbox(&mut game_config.aim_lock, "").changed() {
                    self.send_message(Message::ConfigAimLock(game_config.aim_lock));
                    self.write_game_config(&game_config);
                }

                ui.label("Start Bullet")
                    .on_hover_text("after how many bullets fired in a row the aimbot should start");
                if ui
                    .add(
                        egui::DragValue::new(&mut game_config.start_bullet)
                            .range(0..=10)
                            .speed(0.05),
                    )
                    .changed()
                {
                    self.send_message(Message::ConfigStartBullet(game_config.start_bullet));
                    self.write_game_config(&game_config);
                }
                ui.end_row();

                ui.label("Visibility Check")
                    .on_hover_text("whether to check for player visibility");
                if ui.checkbox(&mut game_config.visibility_check, "").changed() {
                    self.send_message(Message::ConfigVisibilityCheck(game_config.visibility_check));
                    self.write_game_config(&game_config);
                }

                ui.label("FOV")
                    .on_hover_text("how much around the crosshair the aimbot should \"see\"");
                if ui
                    .add(
                        egui::DragValue::new(&mut game_config.fov)
                            .range(0.1..=360.0)
                            .suffix("°")
                            .speed(0.02)
                            .max_decimals(1),
                    )
                    .changed()
                {
                    self.send_message(Message::ConfigFOV(game_config.fov));
                    self.write_game_config(&game_config);
                }
                ui.end_row();

                ui.label("Multibone").on_hover_text(
                    "whether the aimbot should aim at all of the body, or just the head",
                );
                if ui.checkbox(&mut game_config.multibone, "").changed() {
                    self.send_message(Message::ConfigMultibone(game_config.multibone));
                    self.write_game_config(&game_config);
                }

                ui.label("Smooth")
                    .on_hover_text("how much the aimbot input should be smoothed, higher is more");
                if ui
                    .add(
                        egui::DragValue::new(&mut game_config.smooth)
                            .range(1.0..=10.0)
                            .speed(0.02)
                            .max_decimals(1),
                    )
                    .changed()
                {
                    self.send_message(Message::ConfigSmooth(game_config.smooth));
                    self.write_game_config(&game_config);
                }
                ui.end_row();

                ui.label("Enable RCS").on_hover_text(
                    "whether recoil should be compensated when the aimbot is not active",
                );
                if ui.checkbox(&mut game_config.rcs, "").changed() {
                    self.send_message(Message::ConfigEnableRCS(game_config.rcs));
                    self.write_game_config(&game_config);
                }
            });

        *self
            .config
            .games
            .get_mut(&self.config.current_game)
            .unwrap() = game_config.clone();
    }

    fn add_game_status(&mut self, ui: &mut Ui) {
        ui.horizontal_top(|ui| {
            ui.label(
                egui::RichText::new(self.status.string())
                    .line_height(Some(8.0))
                    .color(match self.status {
                        AimbotStatus::Working => Colors::GREEN,
                        AimbotStatus::GameNotStarted => Colors::YELLOW,
                    }),
            );

            let mouse_text = match &self.mouse_status {
                MouseStatus::Working(name) => name,
                MouseStatus::PermissionsRequired => {
                    "mouse input only works when user is in input group"
                }
                MouseStatus::Disconnected => "mouse was disconnected",
                MouseStatus::NoMouseFound => "no mouse was found",
            };
            let color = if let MouseStatus::Working(_) = &self.mouse_status {
                Colors::SUBTEXT
            } else {
                Colors::YELLOW
            };
            ui.label(
                egui::RichText::new(mouse_text)
                    .line_height(Some(8.0))
                    .color(color),
            );
        });
    }

    fn write_game_config(&self, game_config: &AimbotConfig) {
        let mut config = self.config.clone();
        *config.games.get_mut(&self.config.current_game).unwrap() = game_config.clone();
        write_config(&config);
    }
}

impl eframe::App for Gui {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        // makes it more inefficient to force draw 60fps, but else the mouse disconnect message does not show up
        // todo: when update is split into tick and show, put message parsing into tick and force update the ui when message are received
        ctx.request_repaint();

        while let Ok(message) = self.rx.try_recv() {
            match message {
                Message::Status(status) => self.status = status,
                Message::MouseStatus(status) => self.mouse_status = status,
                Message::FrameTime(time) => {
                    self.frame_times.push(time);
                    if self.frame_times.len() >= self.frame_times.capacity() {
                        self.average_frame_time = self.frame_times.iter().sum::<f64>()
                            / self.frame_times.capacity() as f64;
                        self.frame_times.clear();
                    }
                }
                _ => {}
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(6.0, 4.0);
            ui.with_layout(
                Layout::left_to_right(Align::Min).with_main_justify(true),
                |ui| {
                    egui::ComboBox::new("game", "Current Game")
                        .selected_text(self.config.current_game.string())
                        .show_ui(ui, |ui| {
                            for game in Game::iter() {
                                let text = game.string();
                                if ui
                                    .selectable_value(
                                        &mut self.config.current_game,
                                        game.clone(),
                                        text,
                                    )
                                    .clicked()
                                {
                                    self.send_message(Message::ChangeGame(
                                        self.config.current_game.clone(),
                                    ));
                                    write_config(&self.config);
                                }
                            }
                        });

                    ui.with_layout(egui::Layout::right_to_left(Align::Min), |ui| {
                        if ui.button("Report Issues").clicked() {
                            ctx.open_url(egui::OpenUrl {
                                url: String::from("https://github.com/avitran0/deadlocked/issues"),
                                new_tab: false,
                            });
                        }
                    });
                },
            );
            ui.separator();

            self.add_game_status(ui);
            ui.separator();

            self.aimbot_grid(ui);
        });

        let font = egui::FontId::proportional(12.0);
        let text_size = ctx.fonts(|fonts| {
            fonts
                .layout_no_wrap(String::from(VERSION), font.clone(), Color32::WHITE)
                .size()
        });

        ctx.layer_painter(egui::LayerId::background()).text(
            Align2::RIGHT_BOTTOM
                .align_size_within_rect(text_size, ctx.screen_rect().shrink(4.0))
                .max,
            Align2::RIGHT_BOTTOM,
            VERSION,
            font.clone(),
            Colors::SUBTEXT,
        );

        let frame_time = format!("{} us", self.average_frame_time);
        ctx.layer_painter(egui::LayerId::background()).text(
            ctx.screen_rect().left_bottom() + vec2(4.0, -4.0),
            Align2::LEFT_BOTTOM,
            frame_time,
            font,
            Colors::SUBTEXT,
        );
    }
}
