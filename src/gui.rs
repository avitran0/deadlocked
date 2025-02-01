use eframe::egui::{self, vec2, Align, Align2, Color32, Ui};
use std::sync::mpsc;
use strum::IntoEnumIterator;

use crate::{
    color::Colors,
    config::{parse_config, write_config, AimbotStatus, Config, VERSION},
    key_codes::KeyCode,
    message::{Game, Message},
    mouse::MouseStatus,
};

#[derive(PartialEq)]
pub enum Tab {
    Aimbot,
    Triggerbot,
}

pub struct Gui {
    tx: mpsc::Sender<Message>,
    rx: mpsc::Receiver<Message>,
    current_tab: Tab,
    config: Config,
    status: AimbotStatus,
    mouse_status: MouseStatus,
    frame_times: Vec<f64>,
    average_frame_time: f64,
}

impl Gui {
    pub fn new(tx: mpsc::Sender<Message>, rx: mpsc::Receiver<Message>) -> Self {
        // read config
        let config = parse_config();
        let status = AimbotStatus::GameNotStarted;
        let out = Self {
            tx,
            rx,
            current_tab: Tab::Aimbot,
            config,
            status,
            mouse_status: MouseStatus::NoMouseFound,
            frame_times: Vec::with_capacity(50),
            average_frame_time: 0.0,
        };
        write_config(&out.config);
        out
    }

    fn send_config(&self) {
        self.send_message(Message::Config(self.config.get().clone()));
        write_config(&self.config);
    }

    fn send_message(&self, message: Message) {
        self.tx.send(message).expect("aimbot thread died");
    }

    fn aimbot_grid(&mut self, ui: &mut Ui) {
        egui::Grid::new("aimbot")
            .num_columns(4)
            .min_col_width(80.0)
            .show(ui, |ui| {
                ui.label("Enable Aimbot");
                if ui
                    .checkbox(&mut self.config.get_mut().enabled, "")
                    .changed()
                {
                    self.send_config();
                }

                ui.label("Hotkey");
                egui::ComboBox::new("aimbot_hotkey", "")
                    .selected_text(format!("{:?}", self.config.get().hotkey))
                    .show_ui(ui, |ui| {
                        for key_code in KeyCode::iter() {
                            let text = format!("{:?}", &key_code);
                            if ui
                                .selectable_value(&mut self.config.get_mut().hotkey, key_code, text)
                                .clicked()
                            {
                                self.send_config();
                            }
                        }
                    });
                ui.end_row();

                ui.label("Aim Lock");
                if ui
                    .checkbox(&mut self.config.get_mut().aim_lock, "")
                    .changed()
                {
                    self.send_config();
                }

                ui.label("Start Bullet");
                if ui
                    .add(
                        egui::DragValue::new(&mut self.config.get_mut().start_bullet)
                            .range(0..=10)
                            .speed(0.05),
                    )
                    .changed()
                {
                    self.send_config();
                }
                ui.end_row();

                ui.label("Visibility Check");
                if ui
                    .checkbox(&mut self.config.get_mut().visibility_check, "")
                    .changed()
                {
                    self.send_config();
                }

                ui.label("FOV");
                if ui
                    .add(
                        egui::DragValue::new(&mut self.config.get_mut().fov)
                            .range(0.1..=360.0)
                            .suffix("°")
                            .speed(0.02)
                            .max_decimals(1),
                    )
                    .changed()
                {
                    self.send_config();
                }
                ui.end_row();

                ui.label("Multibone");
                if ui
                    .checkbox(&mut self.config.get_mut().multibone, "")
                    .changed()
                {
                    self.send_config();
                }

                ui.label("Smooth");
                if ui
                    .add(
                        egui::DragValue::new(&mut self.config.get_mut().smooth)
                            .range(1.0..=10.0)
                            .speed(0.02)
                            .max_decimals(1),
                    )
                    .changed()
                {
                    self.send_config();
                }
                ui.end_row();

                ui.label("Enable RCS");
                if ui.checkbox(&mut self.config.get_mut().rcs, "").changed() {
                    self.send_config();
                }
            });
    }

    fn triggerbot_grid(&mut self, ui: &mut Ui) {
        egui::Grid::new("triggerbot")
            .num_columns(4)
            .min_col_width(80.0)
            .show(ui, |ui| {
                ui.label("Enable");
                if ui
                    .checkbox(&mut self.config.get_mut().triggerbot, "")
                    .changed()
                {
                    self.send_config();
                }

                ui.label("Hotkey");
                egui::ComboBox::new("triggerbot_hotkey", "")
                    .selected_text(format!("{:?}", self.config.get().triggerbot_hotkey))
                    .show_ui(ui, |ui| {
                        for key_code in KeyCode::iter() {
                            let text = format!("{:?}", &key_code);
                            if ui
                                .selectable_value(
                                    &mut self.config.get_mut().triggerbot_hotkey,
                                    key_code,
                                    text,
                                )
                                .clicked()
                            {
                                self.send_config();
                            }
                        }
                    });
                ui.end_row();

                ui.label("Min Delay").on_hover_text(
                    "the minimum time to fire after an enemy\nis in the crosshair, in milliseconds",
                );
                let end = self.config.get().triggerbot_range.end;
                if ui
                    .add(
                        egui::DragValue::new(&mut self.config.get_mut().triggerbot_range.start)
                            .range(0..=end)
                            .speed(0.2),
                    )
                    .changed()
                {
                    self.send_config();
                }

                ui.label("Max Delay");
                let start = self.config.get().triggerbot_range.start;
                if ui
                    .add(
                        egui::DragValue::new(&mut self.config.get_mut().triggerbot_range.end)
                            .range(start..=1000)
                            .speed(0.2),
                    )
                    .changed()
                {
                    self.send_config();
                }
                ui.end_row();

                ui.label("Visibility Check");
                if ui
                    .checkbox(&mut self.config.get_mut().triggerbot_visibility_check, "")
                    .changed()
                {
                    self.send_config();
                }
            });
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
            ui.horizontal(|ui| {
                egui::ComboBox::new("game", "Game")
                    .selected_text(self.config.current_game.string())
                    .show_ui(ui, |ui| {
                        for game in Game::iter() {
                            let text = game.string();
                            if ui
                                .selectable_value(&mut self.config.current_game, game.clone(), text)
                                .clicked()
                            {
                                self.send_message(Message::ChangeGame(
                                    self.config.current_game.clone(),
                                ));
                                write_config(&self.config);
                            }
                        }
                    });

                ui.selectable_value(&mut self.current_tab, Tab::Aimbot, "Aimbot");
                ui.selectable_value(&mut self.current_tab, Tab::Triggerbot, "Triggerbot");

                ui.with_layout(egui::Layout::right_to_left(Align::Min), |ui| {
                    if ui.button("Report Issues").clicked() {
                        ctx.open_url(egui::OpenUrl {
                            url: String::from("https://github.com/avitran0/deadlocked/issues"),
                            new_tab: false,
                        });
                    }
                });
            });
            ui.separator();

            self.add_game_status(ui);
            ui.separator();

            match self.current_tab {
                Tab::Aimbot => self.aimbot_grid(ui),
                Tab::Triggerbot => self.triggerbot_grid(ui),
            }
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
