use eframe::egui::{
    self, pos2, vec2, Align, Align2, Color32, Layout, Sense, Stroke, Ui, ViewportBuilder,
    ViewportId,
};
use glam::{vec3, Mat4, Vec2, Vec4};
use std::{sync::mpsc, time::Duration};
use strum::IntoEnumIterator;

use crate::{
    color::{Color, Colors},
    config::{parse_config, write_config, AimbotStatus, Config, DrawMode, VisualsConfig, VERSION},
    key_codes::KeyCode,
    math::world_to_screen,
    message::{Message, PlayerInfo},
    mouse::MouseStatus,
};

#[derive(Debug, PartialEq)]
enum Tab {
    Aimbot,
    Triggerbot,
    Visuals,
    Colors,
}

pub struct Gui {
    tx_aimbot: mpsc::Sender<Message>,
    rx: mpsc::Receiver<Message>,
    current_tab: Tab,
    config: Config,
    status: AimbotStatus,
    mouse_status: MouseStatus,
    frame_times: Vec<f64>,
    average_frame_time: f64,
    player_info: Vec<PlayerInfo>,
    view_matrix: Mat4,
    window_info: Vec4,
}

impl Gui {
    pub fn new(tx_aimbot: mpsc::Sender<Message>, rx: mpsc::Receiver<Message>) -> Self {
        // read config
        let config = parse_config();
        let status = AimbotStatus::GameNotStarted;
        let out = Self {
            tx_aimbot,
            rx,
            current_tab: Tab::Aimbot,
            config,
            status,
            mouse_status: MouseStatus::NoMouseFound,
            frame_times: Vec::with_capacity(50),
            average_frame_time: 0.0,
            player_info: vec![],
            view_matrix: Mat4::ZERO,
            window_info: Vec4::ZERO,
        };
        write_config(&out.config);
        out
    }

    fn send_config(&self) {
        self.send_message(Message::AimbotConfig(self.config.aimbot.clone()));
    }

    fn send_message(&self, message: Message) {
        self.tx_aimbot.send(message).expect("aimbot thread died");
        self.save_config();
    }

    fn aimbot_grid(&mut self, ui: &mut Ui) {
        egui::Grid::new("aimbot")
            .num_columns(2)
            .min_col_width(80.0)
            .show(ui, |ui| {
                ui.label("Enable Aimbot")
                    .on_hover_text("general aimbot enable");
                if ui.checkbox(&mut self.config.aimbot.enabled, "").changed() {
                    self.send_config();
                }

                ui.label("Hotkey")
                    .on_hover_text("which key or mouse button should activate the aimbot");
                egui::ComboBox::new("aimbot_hotkey", "")
                    .selected_text(format!("{:?}", self.config.aimbot.hotkey))
                    .show_ui(ui, |ui| {
                        for key_code in KeyCode::iter() {
                            let text = format!("{:?}", &key_code);
                            if ui
                                .selectable_value(&mut self.config.aimbot.hotkey, key_code, text)
                                .clicked()
                            {
                                self.send_config();
                            }
                        }
                    });
                ui.end_row();

                ui.label("Aim Lock")
                    .on_hover_text("whether the aim should lock onto the target");
                if ui.checkbox(&mut self.config.aimbot.aim_lock, "").changed() {
                    self.send_config();
                }

                ui.label("Start Bullet")
                    .on_hover_text("after how many bullets fired in a row the aimbot should start");
                if ui
                    .add(
                        egui::DragValue::new(&mut self.config.aimbot.start_bullet)
                            .range(0..=10)
                            .speed(0.05),
                    )
                    .changed()
                {
                    self.send_config();
                }
                ui.end_row();

                ui.label("Visibility Check")
                    .on_hover_text("whether to check for player visibility");
                if ui
                    .checkbox(&mut self.config.aimbot.visibility_check, "")
                    .changed()
                {
                    self.send_config();
                }

                ui.label("FOV")
                    .on_hover_text("how much around the crosshair the aimbot should \"see\"");
                if ui
                    .add(
                        egui::DragValue::new(&mut self.config.aimbot.fov)
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

                ui.label("Multibone").on_hover_text(
                    "whether the aimbot should aim at all of the body, or just the head",
                );
                if ui.checkbox(&mut self.config.aimbot.multibone, "").changed() {
                    self.send_config();
                }

                ui.label("Smooth")
                    .on_hover_text("how much the aimbot input should be smoothed\nhigher is more");
                if ui
                    .add(
                        egui::DragValue::new(&mut self.config.aimbot.smooth)
                            .range(1.0..=10.0)
                            .speed(0.02)
                            .max_decimals(1),
                    )
                    .changed()
                {
                    self.send_config();
                }
                ui.end_row();

                ui.label("Enable RCS").on_hover_text(
                    "whether recoil should be compensated when the\naimbot is not active",
                );
                if ui.checkbox(&mut self.config.aimbot.rcs, "").changed() {
                    self.send_config();
                }
                ui.end_row();
            });
    }

    fn triggerbot_grid(&mut self, ui: &mut Ui) {
        egui::Grid::new("aimbot")
            .num_columns(2)
            .min_col_width(80.0)
            .show(ui, |ui| {
                ui.label("Enable").on_hover_text("whether to automatically fire when\na player is in the crosshair and the hotkey is held");
                if ui.checkbox(&mut self.config.aimbot.triggerbot, "").changed() {
                    self.send_config();
                }

                    // bad hack to have hotkey on right side
                    ui.label("Hotkey")
                        .on_hover_text("which key or mouse button should activate the triggerbot");
                    egui::ComboBox::new("triggerbot_hotkey", "")
                        .selected_text(format!("{:?}", self.config.aimbot.triggerbot_hotkey))
                        .show_ui(ui, |ui| {
                            for key_code in KeyCode::iter() {
                                let text = format!("{:?}", &key_code);
                                if ui
                                    .selectable_value(
                                        &mut self.config.aimbot.triggerbot_hotkey,
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

                    ui.label("Min Delay").on_hover_text("the minimum time to fire after an enemy\nis in the crosshair, in milliseconds");
                    if ui
                        .add(
                            egui::DragValue::new(&mut self.config.aimbot.triggerbot_range.start)
                                .range(0..=self.config.aimbot.triggerbot_range.end)
                                .speed(0.2),
                        )
                        .changed()
                    {
                        self.send_config();
                    }

                    ui.label("Max Delay").on_hover_text("the maximum time to fire after an enemy\nis in the crosshair, in milliseconds");
                    if ui
                        .add(
                            egui::DragValue::new(&mut self.config.aimbot.triggerbot_range.end)
                                .range(self.config.aimbot.triggerbot_range.start..=1000)
                                .speed(0.2),
                        )
                        .changed()
                    {
                        self.send_config();
                    }
                    ui.end_row();
            });
    }

    fn visuals_grid(&mut self, ui: &mut Ui) {
        egui::Grid::new("aimbot")
            .num_columns(2)
            .min_col_width(80.0)
            .show(ui, |ui| {
                ui.label("Enable");
                if ui.checkbox(&mut self.config.visuals.enabled, "").changed() {
                    self.send_config();
                }

                ui.label("Draw Box");
                egui::ComboBox::new("visuals_draw_box", "")
                    .selected_text(format!("{:?}", self.config.visuals.draw_box))
                    .show_ui(ui, |ui| {
                        for draw_style in DrawMode::iter() {
                            let text = format!("{:?}", &draw_style);
                            if ui
                                .selectable_value(
                                    &mut self.config.visuals.draw_box,
                                    draw_style,
                                    text,
                                )
                                .clicked()
                            {
                                self.send_config();
                            }
                        }
                    });
                ui.end_row();

                ui.label("Draw Health");
                if ui
                    .checkbox(&mut self.config.visuals.draw_health, "")
                    .changed()
                {
                    self.send_config();
                }

                ui.label("Draw Skeleton");
                egui::ComboBox::new("visuals_draw_skeleton", "")
                    .selected_text(format!("{:?}", self.config.visuals.draw_skeleton))
                    .show_ui(ui, |ui| {
                        for draw_style in DrawMode::iter() {
                            let text = format!("{:?}", &draw_style);
                            if ui
                                .selectable_value(
                                    &mut self.config.visuals.draw_skeleton,
                                    draw_style,
                                    text,
                                )
                                .clicked()
                            {
                                self.send_config();
                            }
                        }
                    });
                ui.end_row();

                ui.label("Draw Armor");
                if ui
                    .checkbox(&mut self.config.visuals.draw_armor, "")
                    .changed()
                {
                    self.send_config();
                }

                ui.label("Debug");
                if ui.checkbox(&mut self.config.visuals.debug, "").changed() {
                    self.send_config();
                }
                ui.end_row();
            });
    }

    fn colors_grid(&mut self, ui: &mut Ui) {
        egui::Grid::new("aimbot")
            .num_columns(2)
            .min_col_width(80.0)
            .show(ui, |ui| {
                ui.label("Box Color");
                if let Some(color) = self.color_picker(ui, &self.config.visuals.box_color) {
                    self.config.visuals.box_color = color;
                    self.send_config();
                }
                ui.end_row();

                ui.label("Skeleton Color");
                if let Some(color) = self.color_picker(ui, &self.config.visuals.skeleton_color) {
                    self.config.visuals.skeleton_color = color;
                    self.send_config();
                }
                ui.end_row();
            });
    }

    fn color_picker(&self, ui: &mut Ui, color: &Color) -> Option<Color> {
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

    fn save_config(&self) {
        write_config(&self.config);
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

    fn health_color(&self, player: &PlayerInfo) -> Color {
        let health = player.health.clamp(0, 100);

        let (r, g, b) = if health <= 50 {
            let factor = health as f32 / 50.0;
            (255, (255.0 * factor) as u8, 0)
        } else {
            let factor = (health as f32 - 50.0) / 50.0;
            ((255.0 * (1.0 - factor)) as u8, 255, 0)
        };

        Color::rgb(r, g, b)
    }

    fn player_top_bottom(&self, player: &PlayerInfo) -> Option<(Vec2, Vec2)> {
        let bottom = world_to_screen(&self.window_info, &self.view_matrix, &player.position)?;
        let head = vec3(player.head.x, player.head.y, player.head.z + 8.0);
        let top = world_to_screen(&self.window_info, &self.view_matrix, &head)?;
        let y_delta = top.y - bottom.y;
        Some((bottom, glam::vec2(bottom.x, bottom.y + y_delta)))
    }

    fn draw_box(&self, painter: &egui::Painter, player: &PlayerInfo, config: &VisualsConfig) {
        let color = match &config.draw_box {
            DrawMode::None => return,
            DrawMode::Color => config.box_color,
            DrawMode::Health => self.health_color(player),
        }
        .egui_color();

        let (bottom, top) = match self.player_top_bottom(player) {
            Some(x) => x,
            None => return,
        };

        let width = (top.y - bottom.y) / 2.0;
        let half_width = width / 2.0;
        let line_length = width / 4.0;

        let top_left = pos2(top.x - half_width, top.y);
        let top_right = pos2(top.x + half_width, top.y);
        let bottom_left = pos2(bottom.x - half_width, bottom.y);
        let bottom_right = pos2(bottom.x + half_width, bottom.y);

        let stroke = Stroke::new(1.5, color);

        painter.line(
            vec![
                pos2(top_left.x + line_length, top_left.y),
                top_left,
                top_left,
                pos2(top_left.x, top_left.y - line_length),
            ],
            stroke,
        );

        painter.line(
            vec![
                pos2(top_right.x - line_length, top_right.y),
                top_right,
                top_right,
                pos2(top_right.x, top_right.y - line_length),
            ],
            stroke,
        );

        painter.line(
            vec![
                pos2(bottom_left.x + line_length, bottom_left.y),
                bottom_left,
                bottom_left,
                pos2(bottom_left.x, bottom_left.y + line_length),
            ],
            stroke,
        );

        painter.line(
            vec![
                pos2(bottom_right.x - line_length, bottom_right.y),
                bottom_right,
                bottom_right,
                pos2(bottom_right.x, bottom_right.y + line_length),
            ],
            stroke,
        );
    }

    fn draw_skeleton(&self, painter: &egui::Painter, player: &PlayerInfo, config: &VisualsConfig) {
        let color = match &config.draw_skeleton {
            DrawMode::None => return,
            DrawMode::Color => config.skeleton_color,
            DrawMode::Health => self.health_color(player),
        }
        .egui_color();
        for bones in &player.bones {
            let bone1 = match world_to_screen(&self.window_info, &self.view_matrix, &bones.0) {
                Some(bone) => bone,
                None => continue,
            };
            let bone2 = match world_to_screen(&self.window_info, &self.view_matrix, &bones.1) {
                Some(bone) => bone,
                None => continue,
            };

            painter.line(
                vec![pos2(bone1.x, bone1.y), pos2(bone2.x, bone2.y)],
                Stroke::new(1.5, color),
            );
        }
    }

    fn draw_bars(&self, painter: &egui::Painter, player: &PlayerInfo, config: &VisualsConfig) {
        if !config.draw_health && !config.draw_armor {
            return;
        }

        let (bottom, top) = match self.player_top_bottom(player) {
            Some(x) => x,
            None => return,
        };

        let height = top.y - bottom.y;
        let width = height / 2.0;

        if config.draw_health {
            let bottom_left = pos2(bottom.x + width / 2.0 - 4.0, bottom.y);
            let stroke = Stroke::new(1.0, self.health_color(player).egui_color());
            let health_height = height * (player.health.clamp(0, 100) as f32 / 100.0);
            painter.line(
                vec![
                    bottom_left,
                    pos2(bottom_left.x, bottom_left.y + health_height),
                ],
                stroke,
            );
        }

        if config.draw_armor {
            let bottom_right = pos2(bottom.x - width / 2.0 + 4.0, bottom.y);
            let stroke = Stroke::new(1.0, Color32::BLUE);
            let armor_height = height * (player.armor.clamp(0, 100) as f32 / 100.0);
            painter.line(
                vec![
                    bottom_right,
                    pos2(bottom_right.x, bottom_right.y + armor_height),
                ],
                stroke,
            );
        }
    }
}

impl eframe::App for Gui {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        // makes it more inefficient to force draw 60fps, but else the mouse disconnect message does not show up
        // todo: when update is split into tick and show, put message parsing into tick and force update the ui when message are received
        ctx.request_repaint_after(Duration::from_millis(5));

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
                Message::PlayerInfo(info) => self.player_info = info,
                Message::GameInfo(info) => {
                    self.view_matrix = info.0;
                    self.window_info = info.1
                }
                _ => {}
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(6.0, 4.0);

            // todo: tab selection
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.current_tab, Tab::Aimbot, "Aimbot");
                ui.selectable_value(&mut self.current_tab, Tab::Triggerbot, "Triggerbot");
                ui.selectable_value(&mut self.current_tab, Tab::Visuals, "Visuals");
                ui.selectable_value(&mut self.current_tab, Tab::Colors, "Colors");

                ui.with_layout(
                    Layout::left_to_right(Align::Min).with_main_justify(true),
                    |ui| {
                        ui.with_layout(egui::Layout::right_to_left(Align::Min), |ui| {
                            if ui.button("Report Issues").clicked() {
                                ctx.open_url(egui::OpenUrl {
                                    url: String::from(
                                        "https://github.com/avitran0/deadlocked/issues",
                                    ),
                                    new_tab: false,
                                });
                            }
                        });
                    },
                );
            });

            ui.separator();

            self.add_game_status(ui);
            ui.separator();

            match self.current_tab {
                Tab::Aimbot => self.aimbot_grid(ui),
                Tab::Triggerbot => self.triggerbot_grid(ui),
                Tab::Visuals => self.visuals_grid(ui),
                Tab::Colors => self.colors_grid(ui),
            }

            ctx.show_viewport_immediate(
                ViewportId::from_hash_of("cock"),
                ViewportBuilder::default()
                    .with_always_on_top()
                    .with_decorations(false)
                    .with_mouse_passthrough(true)
                    .with_position((0.0, 0.0))
                    .with_inner_size((8192.0 / 1.5, 8192.0 / 1.5))
                    .with_transparent(true),
                |context, _class| {
                    context.request_repaint_after(Duration::from_millis(5));
                    let painter = context.debug_painter();
                    painter.rect_filled(context.screen_rect(), 0.0, Color32::TRANSPARENT);
                    for player in &self.player_info {
                        self.draw_box(&painter, player, &self.config.visuals);
                        self.draw_skeleton(&painter, player, &self.config.visuals);
                        self.draw_bars(&painter, player, &self.config.visuals);
                    }
                    if self.config.visuals.debug {
                        painter.line(
                            vec![pos2(0.0, 0.0), context.screen_rect().max],
                            Stroke::new(1.5, Colors::TEXT),
                        );
                        painter.line(
                            vec![
                                pos2(0.0, context.screen_rect().max.y),
                                pos2(context.screen_rect().max.x, 0.0),
                            ],
                            Stroke::new(1.5, Colors::TEXT),
                        );
                    }
                },
            )
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

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }
}
