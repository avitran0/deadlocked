use eframe::egui::{self, vec2, Align2, Color32, Sense, Ui};
use std::{cmp::Ordering, sync::mpsc};
use strum::IntoEnumIterator;

use crate::{
    color::{Color, Colors},
    config::{parse_config, write_config, AimbotStatus, Config, GameConfig},
    key_codes::KeyCode,
    message::{AimbotMessage, DrawStyle, Game, VisualsMessage},
    mouse::MouseStatus,
};

#[derive(Debug, PartialEq)]
enum Tab {
    Aimbot,
    Visuals,
}

pub struct Gui {
    tx_aimbot: mpsc::Sender<AimbotMessage>,
    tx_visuals: mpsc::Sender<VisualsMessage>,
    rx: mpsc::Receiver<AimbotMessage>,
    config: Config,
    status: AimbotStatus,
    mouse_status: MouseStatus,
    current_tab: Tab,
    close_timer: i32,

    aimbot_time: f64,
    aimbot_times: Vec<f64>,
}

impl Gui {
    pub fn new(
        tx_aimbot: mpsc::Sender<AimbotMessage>,
        tx_visuals: mpsc::Sender<VisualsMessage>,
        rx: mpsc::Receiver<AimbotMessage>,
    ) -> Self {
        // read config
        let config = parse_config();
        let status = AimbotStatus::GameNotStarted;
        tx_visuals
            .send(VisualsMessage::Config(
                config
                    .games
                    .get(&config.current_game)
                    .unwrap()
                    .visuals
                    .clone(),
            ))
            .unwrap();
        let out = Self {
            tx_aimbot,
            tx_visuals,
            rx,
            config,
            status,
            mouse_status: MouseStatus::NoMouseFound,
            current_tab: Tab::Aimbot,
            close_timer: -1,

            aimbot_time: 0.0,
            aimbot_times: Vec::with_capacity(50),
        };
        write_config(&out.config);
        out
    }

    fn send_message(&self, message: AimbotMessage) {
        self.tx_aimbot.send(message).unwrap();
    }

    fn send_visuals_message(&self, message: VisualsMessage) {
        self.tx_visuals.send(message).unwrap();
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
            .min_col_width(100.0)
            .show(ui, |ui| {
                ui.label("Enable Aimbot")
                    .on_hover_text("general aimbot enable");
                if ui.checkbox(&mut game_config.aimbot.enabled, "").changed() {
                    self.send_message(AimbotMessage::ConfigEnableAimbot(
                        game_config.aimbot.enabled,
                    ));
                    self.write_game_config(&game_config);
                }
                ui.end_row();

                ui.label("Hotkey")
                    .on_hover_text("which key or mouse button should activate the aimbot");
                egui::ComboBox::new("aimbot_hotkey", "")
                    .selected_text(format!("{:?}", game_config.aimbot.hotkey))
                    .show_ui(ui, |ui| {
                        for key_code in KeyCode::iter() {
                            let text = format!("{:?}", &key_code);
                            if ui
                                .selectable_value(&mut game_config.aimbot.hotkey, key_code, text)
                                .clicked()
                            {
                                self.send_message(AimbotMessage::ConfigHotkey(
                                    game_config.aimbot.hotkey,
                                ));
                                self.write_game_config(&game_config);
                            }
                        }
                    });
                ui.end_row();

                ui.label("Start Bullet")
                    .on_hover_text("after how many bullets fired in a row the aimbot should start");
                if ui
                    .add(
                        egui::DragValue::new(&mut game_config.aimbot.start_bullet)
                            .range(0..=10)
                            .speed(0.05),
                    )
                    .changed()
                {
                    self.send_message(AimbotMessage::ConfigStartBullet(
                        game_config.aimbot.start_bullet,
                    ));
                    self.write_game_config(&game_config);
                }
                ui.end_row();

                ui.label("Aim Lock")
                    .on_hover_text("whether the aim should lock onto the target");
                if ui.checkbox(&mut game_config.aimbot.aim_lock, "").changed() {
                    self.send_message(AimbotMessage::ConfigAimLock(game_config.aimbot.aim_lock));
                    self.write_game_config(&game_config);
                }
                ui.end_row();

                ui.label("Visibility Check")
                    .on_hover_text("whether to check for player visibility");
                if ui
                    .checkbox(&mut game_config.aimbot.visibility_check, "")
                    .changed()
                {
                    self.send_message(AimbotMessage::ConfigVisibilityCheck(
                        game_config.aimbot.visibility_check,
                    ));
                    self.write_game_config(&game_config);
                }
                ui.end_row();

                ui.label("FOV")
                    .on_hover_text("how much around the crosshair the aimbot should \"see\"");
                if ui
                    .add(
                        egui::DragValue::new(&mut game_config.aimbot.fov)
                            .range(0.1..=360.0)
                            .suffix("°")
                            .speed(0.02)
                            .max_decimals(1),
                    )
                    .changed()
                {
                    self.send_message(AimbotMessage::ConfigFOV(game_config.aimbot.fov));
                    self.write_game_config(&game_config);
                }
                ui.end_row();

                ui.label("Smooth")
                    .on_hover_text("how much the aimbot input should be smoothed, higher is more");
                if ui
                    .add(
                        egui::DragValue::new(&mut game_config.aimbot.smooth)
                            .range(1.0..=10.0)
                            .speed(0.02)
                            .max_decimals(1),
                    )
                    .changed()
                {
                    self.send_message(AimbotMessage::ConfigSmooth(game_config.aimbot.smooth));
                    self.write_game_config(&game_config);
                }
                ui.end_row();

                ui.label("Multibone").on_hover_text(
                    "whether the aimbot should aim at all of the body, or just the head",
                );
                if ui.checkbox(&mut game_config.aimbot.multibone, "").changed() {
                    self.send_message(AimbotMessage::ConfigMultibone(game_config.aimbot.multibone));
                    self.write_game_config(&game_config);
                }
                ui.end_row();

                ui.label("Enable RCS").on_hover_text(
                    "whether recoil should be compensated when the aimbot is not active",
                );
                if ui.checkbox(&mut game_config.aimbot.rcs, "").changed() {
                    self.send_message(AimbotMessage::ConfigEnableRCS(game_config.aimbot.rcs));
                    self.write_game_config(&game_config);
                }
            });

        *self
            .config
            .games
            .get_mut(&self.config.current_game)
            .unwrap() = game_config.clone();
    }

    fn visuals_grid(&mut self, ui: &mut Ui) {
        let mut game_config = self
            .config
            .games
            .get_mut(&self.config.current_game)
            .unwrap()
            .clone();

        egui::Grid::new("visuals").show(ui, |ui| {
            ui.label("Enable Visuals")
                .on_hover_text("general visuals enable");
            if ui.checkbox(&mut game_config.visuals.enabled, "").changed() {
                self.send_visuals_message(VisualsMessage::EnableVisuals(
                    game_config.visuals.enabled,
                ));
                self.write_game_config(&game_config);
            }
            ui.end_row();

            ui.label("Box")
                .on_hover_text("whether to draw a box, and if so, in which color");
            egui::ComboBox::new("visuals_draw_box", "")
                .selected_text(format!("{:?}", game_config.visuals.draw_box))
                .show_ui(ui, |ui| {
                    for draw_style in DrawStyle::iter() {
                        let text = format!("{:?}", &draw_style);
                        if ui
                            .selectable_value(&mut game_config.visuals.draw_box, draw_style, text)
                            .clicked()
                        {
                            self.send_visuals_message(VisualsMessage::DrawBox(
                                game_config.visuals.draw_box,
                            ));
                            self.write_game_config(&game_config);
                        }
                    }
                });
            if game_config.visuals.draw_box == DrawStyle::Color {
                if let Some(color) = self.color_picker(ui, &mut game_config.visuals.box_color) {
                    game_config.visuals.box_color = color;
                    self.send_visuals_message(VisualsMessage::BoxColor(
                        game_config.visuals.box_color,
                    ));
                    self.write_game_config(&game_config);
                }
            }
            ui.end_row();

            ui.label("Skeleton")
                .on_hover_text("whether to draw player skeletons, and if so, in which color");
            egui::ComboBox::new("visuals_draw_skeleton", "")
                .selected_text(format!("{:?}", game_config.visuals.draw_skeleton))
                .show_ui(ui, |ui| {
                    for draw_style in DrawStyle::iter() {
                        let text = format!("{:?}", &draw_style);
                        if ui
                            .selectable_value(
                                &mut game_config.visuals.draw_skeleton,
                                draw_style,
                                text,
                            )
                            .clicked()
                        {
                            self.send_visuals_message(VisualsMessage::DrawSkeleton(
                                game_config.visuals.draw_skeleton,
                            ));
                            self.write_game_config(&game_config);
                        }
                    }
                });
            if game_config.visuals.draw_skeleton == DrawStyle::Color {
                if let Some(color) = self.color_picker(ui, &mut game_config.visuals.skeleton_color)
                {
                    game_config.visuals.skeleton_color = color;
                    self.send_visuals_message(VisualsMessage::SkeletonColor(
                        game_config.visuals.skeleton_color,
                    ));
                    self.write_game_config(&game_config);
                }
            }
            ui.end_row();

            ui.label("Health Bar")
                .on_hover_text("whether to draw player health\nalways fades from green to red");
            if ui
                .checkbox(&mut game_config.visuals.draw_health, "")
                .changed()
            {
                self.send_visuals_message(VisualsMessage::DrawHealth(
                    game_config.visuals.draw_health,
                ));
                self.write_game_config(&game_config);
            }
            ui.end_row();

            ui.label("Armor Bar")
                .on_hover_text("whether to draw player armor");
            if ui
                .checkbox(&mut game_config.visuals.draw_armor, "")
                .changed()
            {
                self.send_visuals_message(VisualsMessage::DrawArmor(
                    game_config.visuals.draw_armor,
                ));
                self.write_game_config(&game_config);
            }
            if game_config.visuals.draw_armor {
                if let Some(color) = self.color_picker(ui, &mut game_config.visuals.armor_color) {
                    game_config.visuals.armor_color = color;
                    self.send_visuals_message(VisualsMessage::ArmorColor(
                        game_config.visuals.armor_color,
                    ));
                    self.write_game_config(&game_config);
                }
            }
            ui.end_row();

            ui.label("Weapon Icons")
                .on_hover_text("whether to show player weapon icons");
            if ui
                .checkbox(&mut game_config.visuals.draw_weapon, "")
                .changed()
            {
                self.send_visuals_message(VisualsMessage::DrawWeapon(
                    game_config.visuals.draw_weapon,
                ));
                self.write_game_config(&game_config);
            }
            ui.end_row();

            ui.label("Dropped Items/Projectiles")
                .on_hover_text("whether to show dropped items and projectiles");
            if ui
                .checkbox(&mut game_config.visuals.dropped_items, "")
                .changed()
            {
                self.send_visuals_message(VisualsMessage::DroppedItems(
                    game_config.visuals.dropped_items,
                ));
                self.write_game_config(&game_config);
            }
            ui.end_row();

            ui.label("Visibility Check")
                .on_hover_text("whether to draw players only when visible");
            if ui
                .checkbox(&mut game_config.visuals.visibility_check, "")
                .changed()
            {
                self.send_visuals_message(VisualsMessage::VisibilityCheck(
                    game_config.visuals.visibility_check,
                ));
                self.write_game_config(&game_config);
            }
            ui.end_row();

            ui.label("Overlay FPS")
                .on_hover_text("what fps the overlay should run at");
            if ui
                .add(egui::DragValue::new(&mut game_config.visuals.fps).range(30..=240))
                .changed()
            {
                self.send_visuals_message(VisualsMessage::VisualsFps(game_config.visuals.fps));
                self.write_game_config(&game_config);
            }
            ui.end_row();

            ui.label("Overlay Debugging")
                .on_hover_text("whether to draw a frame around the overlay window");
            if ui
                .checkbox(&mut game_config.visuals.debug_window, "")
                .changed()
            {
                self.send_visuals_message(VisualsMessage::DebugWindow(
                    game_config.visuals.debug_window,
                ));
                self.write_game_config(&game_config);
            }
            ui.end_row();
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

    fn color_picker(&self, ui: &mut Ui, color: &mut Color) -> Option<Color> {
        let [mut r, mut g, mut b, _] = color.egui_color().to_array();
        let mut changed = false;
        if ui.add(egui::DragValue::new(&mut r).prefix("r: ")).changed() {
            changed = true;
        }
        if ui.add(egui::DragValue::new(&mut g).prefix("g: ")).changed() {
            changed = true;
        }
        if ui.add(egui::DragValue::new(&mut b).prefix("b: ")).changed() {
            changed = true;
        };
        let (response, painter) = ui.allocate_painter(ui.spacing().interact_size, Sense::hover());
        painter.rect_filled(
            response.rect,
            ui.style().visuals.widgets.inactive.rounding,
            color.egui_color(),
        );
        if changed {
            return Some(Color { r, g, b });
        }
        None
    }

    fn write_game_config(&self, game_config: &GameConfig) {
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
        if ctx.input(|i| i.viewport().close_requested()) && self.close_timer == -1 {
            self.send_message(AimbotMessage::Quit);
            self.send_visuals_message(VisualsMessage::Quit);
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.close_timer = 5;
        }
        match self.close_timer.cmp(&0) {
            Ordering::Greater => self.close_timer -= 1,
            Ordering::Equal => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
            _ => {}
        }

        while let Ok(message) = self.rx.try_recv() {
            match message {
                AimbotMessage::Status(status) => self.status = status,
                AimbotMessage::MouseStatus(status) => self.mouse_status = status,
                AimbotMessage::FrameTime(time) => {
                    self.aimbot_times.push(time);
                    if self.aimbot_times.len() >= self.aimbot_times.capacity() {
                        self.aimbot_time = self.aimbot_times.iter().sum::<f64>()
                            / self.aimbot_times.capacity() as f64;
                        self.aimbot_times.clear();
                    }
                }
                _ => {}
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(6.0, 4.0);
            ui.horizontal_top(|ui| {
                egui::ComboBox::new("game", "Current Game")
                    .selected_text(self.config.current_game.string())
                    .show_ui(ui, |ui| {
                        for game in Game::iter() {
                            let text = game.string();
                            if ui
                                .selectable_value(&mut self.config.current_game, game.clone(), text)
                                .clicked()
                            {
                                self.send_message(AimbotMessage::ChangeGame(
                                    self.config.current_game.clone(),
                                ));
                                self.send_visuals_message(VisualsMessage::Config(
                                    self.config
                                        .games
                                        .get(&self.config.current_game)
                                        .unwrap()
                                        .visuals
                                        .clone(),
                                ));
                                write_config(&self.config);
                            }
                        }
                    });

                ui.add_sized([5.0, 20.0], egui::widgets::Separator::default().vertical());

                ui.selectable_value(&mut self.current_tab, Tab::Aimbot, "Aimbot");
                ui.selectable_value(&mut self.current_tab, Tab::Visuals, "Visuals");

                ui.add_sized([5.0, 20.0], egui::widgets::Separator::default().vertical());

                if ui.button("Report Issues").clicked() {
                    ctx.open_url(egui::OpenUrl {
                        url: String::from("https://github.com/avitran0/deadlocked/issues"),
                        new_tab: false,
                    });
                }
            });

            ui.separator();
            self.add_game_status(ui);
            ui.separator();

            match self.current_tab {
                Tab::Aimbot => self.aimbot_grid(ui),
                Tab::Visuals => self.visuals_grid(ui),
            }
        });

        let version = format!(
            "version: {}",
            option_env!("CARGO_PKG_VERSION").unwrap_or("unknown")
        );
        let font = egui::FontId::proportional(12.0);
        let text_size = ctx.fonts(|fonts| {
            fonts
                .layout_no_wrap(version.clone(), font.clone(), Color32::WHITE)
                .size()
        });

        ctx.layer_painter(egui::LayerId::background()).text(
            Align2::RIGHT_BOTTOM
                .align_size_within_rect(text_size, ctx.screen_rect().shrink(4.0))
                .max,
            Align2::RIGHT_BOTTOM,
            version,
            font.clone(),
            Colors::SUBTEXT,
        );

        let frame_time = format!("{:.2} ms", self.aimbot_time);
        let text_size = ctx.fonts(|fonts| {
            fonts
                .layout_no_wrap(frame_time.clone(), font.clone(), Color32::WHITE)
                .size()
        });

        ctx.layer_painter(egui::LayerId::background()).text(
            Align2::RIGHT_BOTTOM
                .align_size_within_rect(text_size, ctx.screen_rect().shrink(4.0))
                .max - vec2(0.0, 14.0),
            Align2::RIGHT_BOTTOM,
            frame_time,
            font,
            Colors::SUBTEXT,
        );
    }
}
