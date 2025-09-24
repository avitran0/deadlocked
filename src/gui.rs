use egui::{
    Align, Align2, Button, CollapsingHeader, Color32, Context, DragValue, FontId, Painter, Pos2,
    Sense, Stroke, Ui, pos2,
};
use egui_glow::glow;
use glam::{Vec3, vec3};
use strum::IntoEnumIterator;

use crate::{
    app::App,
    bvh::{Aabb, Triangle},
    color::Colors,
    config::{
        AimbotConfig, BoxMode, CONFIG_PATH, Config, DrawMode, TriggerbotMode, VERSION,
        WeaponConfig, available_configs, delete_config, parse_config, write_config,
    },
    constants::cs2,
    cs2::{bones::Bones, weapon::Weapon, weapon_class::WeaponClass},
    data::{Data, PlayerData},
    drag_range::DragRange,
    key_codes::KeyCode,
    math::world_to_screen,
    message::{Envelope, GameStatus, Message, RadarStatus, Target},
    mouse::{DeviceStatus, discover_mice},
};

#[derive(PartialEq)]
pub enum Tab {
    Aimbot,
    Player,
    Hud,
    Radar,
    Unsafe,
    Config,
}

#[derive(PartialEq)]
pub enum AimbotTab {
    Global,
    Weapon,
}

const OUTLINE_WIDTH: f32 = 1.0;
fn outline(pos: Pos2, color: Color32) -> [(Pos2, Color32); 5] {
    [
        (
            pos2(pos.x - OUTLINE_WIDTH, pos.y - OUTLINE_WIDTH),
            Color32::BLACK,
        ),
        (
            pos2(pos.x + OUTLINE_WIDTH, pos.y - OUTLINE_WIDTH),
            Color32::BLACK,
        ),
        (
            pos2(pos.x - OUTLINE_WIDTH, pos.y + OUTLINE_WIDTH),
            Color32::BLACK,
        ),
        (
            pos2(pos.x + OUTLINE_WIDTH, pos.y + OUTLINE_WIDTH),
            Color32::BLACK,
        ),
        (pos, color),
    ]
}

impl App {
    pub fn send_config(&self) {
        self.send_message(Message::Config(Box::new(self.config.clone())), Target::Game);
        self.save();
    }

    pub fn send_message(&self, message: Message, target: Target) {
        self.tx.send(Envelope { target, message }).unwrap();
    }

    fn save(&self) {
        write_config(&self.config, &self.current_config);
    }

    fn gui(&mut self, ctx: &Context) {
        ctx.set_pixels_per_point(self.display_scale);
        egui::SidePanel::left("sidebar")
            .resizable(false)
            .show(ctx, |ui| {
                ui.selectable_value(&mut self.current_tab, Tab::Aimbot, "\u{f04fe} Aimbot");
                ui.selectable_value(&mut self.current_tab, Tab::Player, "\u{f0013} Player");
                ui.selectable_value(&mut self.current_tab, Tab::Hud, "\u{f0379} Hud");
                ui.selectable_value(&mut self.current_tab, Tab::Radar, "\u{f0437} Radar");
                ui.selectable_value(&mut self.current_tab, Tab::Unsafe, "\u{f0ce6} Unsafe");
                ui.selectable_value(&mut self.current_tab, Tab::Config, "\u{f168b} Config");

                ui.with_layout(egui::Layout::bottom_up(Align::Min), |ui| {
                    if ui.button("Report Issue").clicked() {
                        std::process::Command::new("xdg-open")
                            .arg("https://github.com/avitran0/deadlocked/issues")
                            .status()
                            .unwrap();
                    }
                    ui.label(VERSION);
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.add_game_status(ui);
            ui.separator();

            match self.current_tab {
                Tab::Aimbot => self.aimbot_settings(ui),
                Tab::Player => self.player_settings(ui),
                Tab::Hud => self.hud_settings(ui),
                Tab::Radar => self.radar_settings(ui, ctx),
                Tab::Unsafe => self.unsafe_settings(ui),
                Tab::Config => self.config_settings(ui, ctx),
            }
        });
    }

    fn aimbot_config(&self, weapon: &Weapon) -> &AimbotConfig {
        if let Some(weapon_config) = self.config.aim.weapons.get(weapon)
            && weapon_config.aimbot.enable_override
        {
            return &weapon_config.aimbot;
        }
        &self.config.aim.global.aimbot
    }

    fn weapon_config(&mut self) -> &mut WeaponConfig {
        if self.aimbot_tab == AimbotTab::Weapon {
            self.config
                .aim
                .weapons
                .get_mut(&self.aimbot_weapon)
                .unwrap()
        } else {
            &mut self.config.aim.global
        }
    }

    fn aimbot_settings(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.aimbot_tab, AimbotTab::Global, "Global");
            ui.selectable_value(&mut self.aimbot_tab, AimbotTab::Weapon, "Weapon");
            if self.aimbot_tab == AimbotTab::Weapon {
                egui::ComboBox::new("aimbot_weapon", "Weapon")
                    .selected_text(format!("{:?}", self.aimbot_weapon))
                    .show_ui(ui, |ui| {
                        for weapon in Weapon::iter() {
                            if weapon == Weapon::Unknown {
                                continue;
                            }
                            let text = format!("{:?}", weapon);
                            ui.selectable_value(&mut self.aimbot_weapon, weapon, text);
                        }
                    });
            }
        });
        ui.separator();
        ui.columns(2, |cols| {
            let left = &mut cols[0];
            egui::ScrollArea::vertical()
                .auto_shrink([false, true])
                .id_salt("aimbot_left")
                .show(left, |left| {
                    self.aimbot_left(left);
                });

            let right = &mut cols[1];
            egui::ScrollArea::vertical()
                .auto_shrink([false, true])
                .id_salt("aimbot_right")
                .show(right, |right| {
                    self.aimbot_right(right);
                });
        });
    }

    fn aimbot_left(&mut self, ui: &mut Ui) {
        collapsing_open(ui, "Aimbot", |ui| {
            egui::ComboBox::new("aimbot_hotkey", "Hotkey")
                .selected_text(format!("{:?}", self.config.aim.hotkey))
                .show_ui(ui, |ui| {
                    for key_code in KeyCode::iter() {
                        let text = format!("{:?}", &key_code);
                        if ui
                            .selectable_value(&mut self.config.aim.hotkey, key_code, text)
                            .clicked()
                        {
                            self.send_config();
                        }
                    }
                });

            if self.aimbot_tab == AimbotTab::Weapon
                && ui
                    .checkbox(
                        &mut self.weapon_config().aimbot.enable_override,
                        "Enable Override",
                    )
                    .changed()
            {
                self.send_config();
            }

            if ui
                .checkbox(&mut self.weapon_config().aimbot.enabled, "Enable Aimbot")
                .changed()
            {
                self.send_config();
            }

            ui.horizontal(|ui| {
                if ui
                    .add(
                        DragValue::new(&mut self.weapon_config().aimbot.fov)
                            .range(0.1..=360.0)
                            .suffix("°")
                            .speed(0.02)
                            .max_decimals(1),
                    )
                    .changed()
                {
                    self.send_config();
                }
                ui.label("FOV");
            });

            ui.horizontal(|ui| {
                if ui
                    .add(
                        DragValue::new(&mut self.weapon_config().aimbot.smooth)
                            .range(0.0..=10.0)
                            .speed(0.02)
                            .max_decimals(1),
                    )
                    .changed()
                {
                    self.send_config();
                }
                ui.label("Smooth");
            });

            ui.horizontal(|ui| {
                if ui
                    .add(
                        DragValue::new(&mut self.weapon_config().aimbot.start_bullet)
                            .range(0..=10)
                            .speed(0.05),
                    )
                    .changed()
                {
                    self.send_config();
                }
                ui.label("Start Bullet");
            });
        });

        ui.collapsing("Checks", |ui| {
            if ui
                .checkbox(
                    &mut self.weapon_config().aimbot.visibility_check,
                    "Visibility Check",
                )
                .changed()
            {
                self.send_config();
            }

            if ui
                .checkbox(&mut self.weapon_config().aimbot.flash_check, "Flash Check")
                .changed()
            {
                self.send_config();
            }
        });

        ui.collapsing("Bones", |ui| {
            for bone in Bones::iter() {
                let text = format!("{:?}", bone);
                let index = self
                    .weapon_config()
                    .aimbot
                    .bones
                    .iter()
                    .position(|b| *b == bone);
                if ui.selectable_label(index.is_some(), text).clicked() {
                    if let Some(index) = index {
                        self.weapon_config().aimbot.bones.remove(index);
                    } else {
                        self.weapon_config().aimbot.bones.push(bone);
                    }
                    self.send_config();
                }
            }
        });
    }

    fn aimbot_right(&mut self, ui: &mut Ui) {
        collapsing_open(ui, "Triggerbot", |ui| {
            if self.aimbot_tab == AimbotTab::Weapon
                && ui
                    .checkbox(
                        &mut self.weapon_config().triggerbot.enable_override,
                        "Enable Override",
                    )
                    .changed()
            {
                self.send_config();
            }

            if ui
                .checkbox(
                    &mut self.weapon_config().triggerbot.enabled,
                    "Enable Triggerbot",
                )
                .changed()
            {
                self.send_config();
            }

            egui::ComboBox::new("triggerbot_hotkey", "Hotkey")
                .selected_text(format!("{:?}", self.config.aim.triggerbot_hotkey))
                .show_ui(ui, |ui| {
                    for key_code in KeyCode::iter() {
                        let text = format!("{:?}", &key_code);
                        if ui
                            .selectable_value(
                                &mut self.config.aim.triggerbot_hotkey,
                                key_code,
                                text,
                            )
                            .clicked()
                        {
                            self.send_config();
                        }
                    }
                });

            ui.horizontal(|ui| {
                if ui
                    .add(DragRange::new(
                        &mut self.weapon_config().triggerbot.delay,
                        0..=999,
                    ))
                    .changed()
                {
                    self.send_config();
                }
                ui.label("Delay (ms)");
            });

            egui::ComboBox::new("triggerbot_mode", "Mode")
                .selected_text(format!("{:?}", self.weapon_config().triggerbot.mode))
                .show_ui(ui, |ui| {
                    for mode in TriggerbotMode::iter() {
                        let text = format!("{:?}", &mode);
                        if ui
                            .selectable_value(&mut self.weapon_config().triggerbot.mode, mode, text)
                            .clicked()
                        {
                            self.send_config();
                        }
                    }
                });

            if ui
                .checkbox(&mut self.weapon_config().triggerbot.head_only, "Head Only")
                .changed()
            {
                self.send_config();
            }

            ui.horizontal(|ui| {
                if ui
                    .add(
                        DragValue::new(&mut self.weapon_config().triggerbot.additional_shots)
                            .range(0..=10)
                            .speed(1.0),
                    )
                    .changed()
                {
                    self.send_config();
                }
                ui.label("Additional Shots");
            });
        });

        ui.collapsing("Checks\u{200b}", |ui| {
            if ui
                .checkbox(
                    &mut self.weapon_config().triggerbot.flash_check,
                    "Flash Check",
                )
                .changed()
            {
                self.send_config();
            }

            if ui
                .checkbox(
                    &mut self.weapon_config().triggerbot.scope_check,
                    "Scope Check",
                )
                .changed()
            {
                self.send_config();
            }

            if ui
                .checkbox(
                    &mut self.weapon_config().triggerbot.velocity_check,
                    "Velocity Check",
                )
                .changed()
            {
                self.send_config();
            }

            ui.horizontal(|ui| {
                if ui
                    .add(
                        DragValue::new(&mut self.weapon_config().triggerbot.velocity_threshold)
                            .range(0..=5000),
                    )
                    .changed()
                {
                    self.send_config();
                }
                ui.label("Velocity Threshold");
            });
        });

        collapsing_open(ui, "RCS", |ui| {
            if self.aimbot_tab == AimbotTab::Weapon
                && ui
                    .checkbox(
                        &mut self.weapon_config().rcs.enable_override,
                        "Enable Override",
                    )
                    .changed()
            {
                self.send_config();
            }

            if ui
                .checkbox(&mut self.weapon_config().rcs.enabled, "Enable RCS")
                .changed()
            {
                self.send_config();
            }

            ui.horizontal(|ui| {
                if ui
                    .add(
                        DragValue::new(&mut self.weapon_config().rcs.smooth)
                            .range(0.0..=1.0)
                            .speed(0.02),
                    )
                    .changed()
                {
                    self.send_config();
                }
                ui.label("RCS Smooth");
            });
        });
    }

    fn player_settings(&mut self, ui: &mut Ui) {
        egui::ScrollArea::vertical()
            .auto_shrink([false, true])
            .id_salt("player")
            .show(ui, |ui| {
                ui.columns(2, |cols| {
                    let left = &mut cols[0];
                    self.player_left(left);
                    let right = &mut cols[1];
                    self.player_right(right);
                });

                collapsing_open(ui, "Colors", |ui| {
                    if let Some(color) = self.color_picker(
                        ui,
                        &self.config.player.box_visible_color,
                        "Box (visible)",
                    ) {
                        self.config.player.box_visible_color = color;
                        self.send_config();
                    }

                    if let Some(color) = self.color_picker(
                        ui,
                        &self.config.player.box_invisible_color,
                        "Box (invisible)",
                    ) {
                        self.config.player.box_invisible_color = color;
                        self.send_config();
                    }

                    if let Some(color) =
                        self.color_picker(ui, &self.config.player.skeleton_color, "Skeleton")
                    {
                        self.config.player.skeleton_color = color;
                        self.send_config();
                    }

                    if ui
                        .checkbox(&mut self.config.player.head_circle, "Head Circle")
                        .changed()
                    {
                        self.send_config();
                    }

                    ui.horizontal(|ui| {
                        if ui
                            .add(
                                DragValue::new(&mut self.config.player.alpha)
                                    .range(0.0..=1.0)
                                    .speed(0.01)
                                    .max_decimals(2),
                            )
                            .changed()
                        {
                            self.send_config();
                        }
                        ui.label("Alpha (0=transparent, 1=opaque)");
                    });
                });
            });
    }

    fn player_left(&mut self, ui: &mut Ui) {
        collapsing_open(ui, "Players", |ui| {
            if ui
                .checkbox(&mut self.config.player.enabled, "Enable")
                .changed()
            {
                self.send_config();
            }

            egui::ComboBox::new("draw_box", "Box")
                .selected_text(format!("{:?}", self.config.player.draw_box))
                .show_ui(ui, |ui| {
                    for mode in DrawMode::iter() {
                        let text = format!("{:?}", &mode);
                        if ui
                            .selectable_value(&mut self.config.player.draw_box, mode, text)
                            .clicked()
                        {
                            self.send_config();
                        }
                    }
                });

            egui::ComboBox::new("box_mode", "Box Mode")
                .selected_text(format!("{:?}", self.config.player.box_mode))
                .show_ui(ui, |ui| {
                    for mode in BoxMode::iter() {
                        let text = format!("{:?}", &mode);
                        if ui
                            .selectable_value(&mut self.config.player.box_mode, mode, text)
                            .clicked()
                        {
                            self.send_config();
                        }
                    }
                });

            egui::ComboBox::new("draw_skeleton", "Skeleton")
                .selected_text(format!("{:?}", self.config.player.draw_skeleton))
                .show_ui(ui, |ui| {
                    for mode in DrawMode::iter() {
                        let text = format!("{:?}", &mode);
                        if ui
                            .selectable_value(&mut self.config.player.draw_skeleton, mode, text)
                            .clicked()
                        {
                            self.send_config();
                        }
                    }
                });
        });
    }

    fn player_right(&mut self, ui: &mut Ui) {
        collapsing_open(ui, "Info", |ui| {
            if ui
                .checkbox(&mut self.config.player.health_bar, "Health Bar")
                .changed()
            {
                self.send_config();
            }

            if ui
                .checkbox(&mut self.config.player.armor_bar, "Armor Bar")
                .changed()
            {
                self.send_config();
            }

            if ui
                .checkbox(&mut self.config.player.player_name, "Player Name")
                .changed()
            {
                self.send_config();
            }

            if ui
                .checkbox(&mut self.config.player.weapon_icon, "Weapon Icon")
                .changed()
            {
                self.send_config();
            }

            if ui
                .checkbox(&mut self.config.player.tags, "Show Tags")
                .changed()
            {
                self.send_config();
            }
        });
    }

    fn hud_settings(&mut self, ui: &mut Ui) {
        egui::ScrollArea::vertical()
            .auto_shrink([false, true])
            .id_salt("hud")
            .show(ui, |ui| {
                ui.columns(2, |cols| {
                    let left = &mut cols[0];
                    self.hud_left(left);
                    let right = &mut cols[1];
                    self.hud_right(right);
                });

                collapsing_open(ui, "Colors", |ui| {
                    if let Some(color) =
                        self.color_picker(ui, &self.config.hud.text_color, "Text Color")
                    {
                        self.config.hud.text_color = color;
                        self.send_config();
                    }

                    if let Some(color) =
                        self.color_picker(ui, &self.config.hud.crosshair_color, "Crosshair Color")
                    {
                        self.config.hud.crosshair_color = color;
                        self.send_config();
                    }
                });
            });
    }

    fn hud_left(&mut self, ui: &mut Ui) {
        collapsing_open(ui, "HUD", |ui| {
            if ui
                .checkbox(&mut self.config.hud.bomb_timer, "Bomb Timer")
                .changed()
            {
                self.send_config();
            }

            if ui
                .checkbox(&mut self.config.hud.fov_circle, "FOV Circle")
                .changed()
            {
                self.send_config();
            }

            if ui
                .checkbox(&mut self.config.hud.sniper_crosshair, "Sniper Crosshair")
                .changed()
            {
                self.send_config();
            }

            if ui
                .checkbox(&mut self.config.hud.dropped_weapons, "Dropped Weapons")
                .changed()
            {
                self.send_config();
            }

            if ui
                .checkbox(&mut self.config.hud.spectator_list, "Spectator List")
                .changed()
            {
                self.send_config();
            }
        });
    }

    fn hud_right(&mut self, ui: &mut Ui) {
        collapsing_open(ui, "Appearance", |ui| {
            if ui
                .checkbox(&mut self.config.hud.text_outline, "Text Outline")
                .changed()
            {
                self.send_config();
            }

            ui.horizontal(|ui| {
                if ui
                    .add(
                        DragValue::new(&mut self.config.hud.line_width)
                            .range(0.1..=8.0)
                            .speed(0.02)
                            .max_decimals(1),
                    )
                    .changed()
                {
                    self.send_config();
                }
                ui.label("Line Width");
            });

            ui.horizontal(|ui| {
                if ui
                    .add(
                        DragValue::new(&mut self.config.hud.font_size)
                            .range(1.0..=99.0)
                            .speed(0.2)
                            .max_decimals(1),
                    )
                    .changed()
                {
                    self.send_config();
                }
                ui.label("Font Size");
            });
        });

        ui.collapsing("Advanced", |ui| {
            if ui
                .checkbox(&mut self.config.hud.debug, "Debug Overlay")
                .changed()
            {
                self.send_config();
            }
        });
    }

    fn radar_settings(&mut self, ui: &mut Ui, ctx: &Context) {
        egui::ScrollArea::vertical()
            .auto_shrink([false, true])
            .id_salt("hud_left")
            .show(ui, |ui| {
                collapsing_open(ui, "Radar", |ui| {
                    ui.label(egui::RichText::new(format!("{}", self.radar_status)).color(
                        match self.radar_status {
                            RadarStatus::Connected(_) => Colors::GREEN,
                            RadarStatus::Disconnected => Colors::YELLOW,
                        },
                    ));

                    if ui
                        .checkbox(&mut self.config.radar.enabled, "Enable Radar")
                        .changed()
                    {
                        self.send_message(
                            Message::RadarSetEnabled(self.config.radar.enabled),
                            Target::Radar,
                        );
                        self.save();
                    }

                    if ui
                        .text_edit_singleline(&mut self.config.radar.url)
                        .changed()
                    {
                        self.send_message(
                            Message::ChangeRadarUrl(self.config.radar.url.clone()),
                            Target::Radar,
                        );
                        self.save();
                    }

                    if let RadarStatus::Connected(uuid) = &self.radar_status {
                        if ui.button("Open").clicked() {
                            let link = format!("http://{}/?uuid={}", self.config.radar.url, uuid);
                            std::process::Command::new("xdg-open")
                                .arg(&link)
                                .status()
                                .unwrap();
                            log::info!("opened link ({link})");
                        }

                        if ui.button("Copy Link").clicked() {
                            let link = format!("http://{}/?uuid={}", self.config.radar.url, uuid);
                            log::info!("copied link ({link})");
                            ctx.copy_text(link);
                        }
                    }
                });
            });
    }

    fn unsafe_settings(&mut self, ui: &mut Ui) {
        ui.columns(2, |cols| {
            let left = &mut cols[0];
            egui::ScrollArea::vertical()
                .auto_shrink([false, true])
                .id_salt("unsafe_left")
                .show(left, |left| {
                    self.unsafe_left(left);
                });

            let right = &mut cols[1];
            egui::ScrollArea::vertical()
                .auto_shrink([false, true])
                .id_salt("unsafe_right")
                .show(right, |right| {
                    self.unsafe_right(right);
                });
        });

        collapsing_open(ui, "Smokes", |ui| {
            if ui
                .checkbox(&mut self.config.misc.no_smoke, "No Smoke")
                .changed()
            {
                self.send_config();
            }

            if ui
                .checkbox(
                    &mut self.config.misc.change_smoke_color,
                    "Change Smoke Color",
                )
                .changed()
            {
                self.send_config();
            }

            if let Some(color) = self.color_picker(ui, &self.config.misc.smoke_color, "Smoke Color")
            {
                self.config.misc.smoke_color = color;
                self.send_config();
            }
        });
    }

    fn unsafe_left(&mut self, ui: &mut Ui) {
        collapsing_open(ui, "No Flash", |ui| {
            if ui
                .checkbox(&mut self.config.misc.no_flash, "No Flash")
                .changed()
            {
                self.send_config();
            }

            ui.horizontal(|ui| {
                if ui
                    .add(
                        DragValue::new(&mut self.config.misc.max_flash_alpha)
                            .range(0.0..=255.0)
                            .speed(0.5)
                            .max_decimals(0),
                    )
                    .changed()
                {
                    self.send_config();
                }
                ui.label("Max Flash Alpha");
            });
        });
    }

    fn unsafe_right(&mut self, ui: &mut Ui) {
        collapsing_open(ui, "FOV Changer", |ui| {
            if ui
                .checkbox(&mut self.config.misc.fov_changer, "FOV Changer")
                .changed()
            {
                self.send_config();
            }

            ui.horizontal(|ui| {
                if ui
                    .add(
                        DragValue::new(&mut self.config.misc.desired_fov)
                            .speed(0.1)
                            .range(1..=179),
                    )
                    .changed()
                {
                    self.send_config();
                }
                ui.label("Desired FOV");

                if ui.button("Reset").clicked() {
                    self.config.misc.desired_fov = cs2::DEFAULT_FOV;
                    self.send_config();
                }
            });
        });
    }

    fn config_settings(&mut self, ui: &mut Ui, ctx: &Context) {
        ui.columns(2, |cols| {
            let left = &mut cols[0];
            egui::ScrollArea::vertical()
                .auto_shrink([false, true])
                .id_salt("config_left")
                .show(left, |left| {
                    self.config_left(left, ctx);
                });

            let right = &mut cols[1];

            collapsing_open(right, "Configs", |right| {
                right.horizontal(|right| {
                    if right.button("+").clicked() && !self.new_config_name.is_empty() {
                        if !self.new_config_name.ends_with(".toml") {
                            self.new_config_name.push_str(".toml");
                        }
                        let path = CONFIG_PATH.join(&self.new_config_name);
                        write_config(&self.config, &path);
                        self.new_config_name.clear();
                        self.current_config = path;
                        self.available_configs = available_configs();
                    }
                    right.text_edit_singleline(&mut self.new_config_name);
                });

                egui::ScrollArea::vertical()
                    .auto_shrink([false, true])
                    .id_salt("config_right")
                    .show(right, |right| {
                        self.config_right(right);
                    });
            });
        });
    }

    fn config_left(&mut self, ui: &mut Ui, ctx: &Context) {
        collapsing_open(ui, "Config", |ui| {
            if ui.button("Reset").clicked() {
                self.config = Config::default();
                self.send_config();
                log::info!("loaded default config");
            }
        });

        collapsing_open(ui, "Accent Color", |ui| {
            egui::ComboBox::new("accent_color", "Accent Color")
                .selected_text(
                    Colors::ACCENT_COLORS
                        .iter()
                        .find(|c| c.1 == self.config.accent_color)
                        .unwrap_or(&Colors::ACCENT_COLORS[5])
                        .0,
                )
                .show_ui(ui, |ui| {
                    for (name, color) in Colors::ACCENT_COLORS {
                        if ui
                            .add(
                                Button::selectable(color == self.config.accent_color, name)
                                    .fill(color),
                            )
                            .clicked()
                        {
                            self.config.accent_color = color;
                            ctx.style_mut(|style| style.visuals.selection.bg_fill = color);
                        }
                    }
                });
        });
    }

    fn config_right(&mut self, ui: &mut Ui) {
        let mut clicked_config = None;
        let mut delete = None;

        for config in &self.available_configs {
            ui.horizontal(|ui| {
                if ui
                    .add(Button::selectable(
                        *config == self.current_config,
                        config.file_name().unwrap().to_str().unwrap(),
                    ))
                    .clicked()
                {
                    clicked_config = Some(config.clone());
                }
                ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                    if ui.button("\u{f0a7a}").clicked() {
                        delete = Some(config.clone());
                    }
                });
            });
        }

        if let Some(config_path) = clicked_config {
            self.config = parse_config(&config_path);
            self.current_config = config_path;
            self.send_config();
        }

        if let Some(config) = delete {
            delete_config(&config);
            self.available_configs = available_configs();
            self.current_config = self.available_configs[0].clone();
            self.config = parse_config(&self.current_config);
        }
    }

    fn add_game_status(&mut self, ui: &mut Ui) {
        ui.horizontal_top(|ui| {
            ui.label(
                egui::RichText::new(format!("{}", self.game_status))
                    .line_height(Some(8.0))
                    .color(match self.game_status {
                        GameStatus::Working => Colors::GREEN,
                        GameStatus::GameNotStarted => Colors::YELLOW,
                    }),
            );

            let mouse_text = match &self.mouse_status {
                DeviceStatus::Working(name) => name,
                DeviceStatus::PermissionsRequired => {
                    "mouse input only works when user is in input group"
                }
                DeviceStatus::Disconnected => "mouse was disconnected",
                DeviceStatus::NotFound => "no mouse was found",
            };

            let color = match &self.mouse_status {
                DeviceStatus::Working(_) => Colors::SUBTEXT,
                _ => Colors::YELLOW,
            };
            ui.label(
                egui::RichText::new(mouse_text)
                    .line_height(Some(8.0))
                    .color(color),
            );

            egui::ComboBox::new("mouse_device", "")
                .selected_text(
                    self.selected_mouse
                        .as_deref()
                        .unwrap_or("No device selected"),
                )
                .show_ui(ui, |ui| {
                    for device in discover_mice() {
                        let label = format!("{} ({})", device.name, device.event_name);
                        if ui
                            .selectable_label(
                                self.selected_mouse.as_deref() == Some(&device.event_name),
                                &label,
                            )
                            .clicked()
                        {
                            self.selected_mouse = Some(device.event_name.clone());

                            self.send_message(
                                Message::SelectMouse(device.event_name.clone()),
                                Target::Game,
                            );
                        }
                    }
                });
        });
    }

    fn color_picker(&self, ui: &mut Ui, color: &Color32, label: &str) -> Option<Color32> {
        let [mut r, mut g, mut b, _] = color.to_array();
        let res = ui
            .horizontal(|ui| {
                let (response, painter) =
                    ui.allocate_painter(ui.spacing().interact_size, Sense::hover());
                painter.rect_filled(
                    response.rect,
                    ui.style().visuals.widgets.inactive.corner_radius,
                    *color,
                );
                let mut res = ui.add(DragValue::new(&mut r).prefix("r: "));
                res = res.union(ui.add(DragValue::new(&mut g).prefix("g: ")));
                res = res.union(ui.add(DragValue::new(&mut b).prefix("b: ")));
                ui.label(label);
                res
            })
            .inner;

        if res.changed() {
            Some(Color32::from_rgb(r, g, b))
        } else {
            None
        }
    }

    fn apply_alpha(&self, color: Color32) -> Color32 {
        let [r, g, b, _] = color.to_array();
        let alpha = (255.0 * self.config.player.alpha) as u8;
        Color32::from_rgba_unmultiplied(r, g, b, alpha)
    }

    fn overlay(&self, ctx: &Context) {
        ctx.set_pixels_per_point(1.0);
        let painter = ctx.layer_painter(egui::LayerId::background());

        let data = &self.data.lock().unwrap();
        if let Some(window) = &self.overlay_window {
            window
                .window()
                .set_outer_position(winit::dpi::PhysicalPosition::new(
                    data.window_position.x,
                    data.window_position.y,
                ));
            let _ = window
                .window()
                .request_inner_size(winit::dpi::PhysicalSize::new(
                    data.window_size.x.max(1.0),
                    data.window_size.y.max(1.0),
                ));
        }

        if self.config.hud.debug {
            painter.line(
                vec![pos2(0.0, 0.0), pos2(data.window_size.x, data.window_size.y)],
                Stroke::new(self.config.hud.line_width, self.apply_alpha(Color32::WHITE)),
            );
            painter.line(
                vec![pos2(data.window_size.x, 0.0), pos2(0.0, data.window_size.y)],
                Stroke::new(self.config.hud.line_width, self.apply_alpha(Color32::WHITE)),
            );
        }

        if self.config.player.enabled {
            for player in &data.players {
                self.player_box(&painter, player, data);
                self.skeleton(&painter, player, data);
            }
        }

        if self.config.hud.dropped_weapons {
            for weapon in &data.weapons {
                let Some(pos) = world_to_screen(&weapon.1, data) else {
                    continue;
                };
                self.text(
                    &painter,
                    format!("{}", weapon.0),
                    pos,
                    Align2::CENTER_CENTER,
                    None,
                );
            }
        }

        if self.config.hud.bomb_timer && data.bomb.planted {
            if let Some(pos) = world_to_screen(&data.bomb.position, data) {
                self.text(
                    &painter,
                    format!("{:.1}", data.bomb.timer),
                    pos,
                    Align2::CENTER_CENTER,
                    None,
                );
                if data.bomb.being_defused {
                    self.text(
                        &painter,
                        "defusing",
                        pos2(pos.x, pos.y + self.config.hud.font_size),
                        Align2::CENTER_CENTER,
                        None,
                    );
                }
            }

            let fraction = (data.bomb.timer / 40.0).clamp(0.0, 1.0);
            let color = self.apply_alpha(self.health_color((fraction * 100.0) as i32));
            painter.line(
                vec![
                    pos2(0.0, data.window_size.y),
                    pos2(data.window_size.x * fraction, data.window_size.y),
                ],
                Stroke::new(self.config.hud.line_width * 3.0, color),
            );
        }

        // fov circle
        if self.config.hud.fov_circle && data.in_game {
            let weapon_config = self.aimbot_config(&data.weapon);
            let aim_fov = weapon_config.fov;
            let fov = if self.config.misc.fov_changer {
                self.config.misc.desired_fov
            } else {
                cs2::DEFAULT_FOV
            } as f32;
            let radius = (aim_fov.to_radians() / 2.0).tan() / (fov.to_radians() / 2.0).tan()
                * data.window_size.x
                / 2.0;
            painter.circle_stroke(
                pos2(data.window_size.x / 2.0, data.window_size.y / 2.0),
                radius,
                Stroke::new(self.config.hud.line_width, self.apply_alpha(Color32::WHITE)),
            );
        }

        if self.config.hud.sniper_crosshair
            && WeaponClass::from_string(data.weapon.as_ref()) == WeaponClass::Sniper
        {
            painter.line(
                vec![
                    pos2(data.window_size.x / 2.0, data.window_size.y / 2.0 - 50.0),
                    pos2(data.window_size.x / 2.0, data.window_size.y / 2.0 + 50.0),
                ],
                Stroke::new(self.config.hud.line_width, self.apply_alpha(Color32::WHITE)),
            );
            painter.line(
                vec![
                    pos2(data.window_size.x / 2.0 - 50.0, data.window_size.y / 2.0),
                    pos2(data.window_size.x / 2.0 + 50.0, data.window_size.y / 2.0),
                ],
                Stroke::new(self.config.hud.line_width, self.apply_alpha(Color32::WHITE)),
            );
        }

        if data.triggerbot_active {
            self.text(
                &painter,
                "trigger active",
                pos2(
                    data.window_size.x / 2.0 + 8.0,
                    data.window_size.y / 2.0 + 8.0,
                ),
                Align2::LEFT_TOP,
                None,
            );
        }

        if self.config.hud.spectator_list {
            let mut offset = 0.0;
            let half_height = data.window_size.y / 2.0;
            for (player, target) in &data.spectators {
                let Some(player) = find_player(*player, data) else {
                    continue;
                };
                let Some(target) = find_player(*target, data) else {
                    continue;
                };

                let color = if target.steam_id == data.local_player.steam_id {
                    Color32::RED
                } else {
                    self.config.hud.text_color
                };

                self.text(
                    &painter,
                    format!("{} -> {}", player.name, target.name),
                    pos2(4.0, half_height + offset),
                    Align2::LEFT_TOP,
                    Some(color),
                );
                offset += self.config.hud.font_size;
            }
        }
    }

    #[allow(unused)]
    fn triangle(&self, triangle: &Triangle, data: &Data, painter: &Painter) {
        let Some(v1) = world_to_screen(&triangle.v0, data) else {
            return;
        };
        let Some(v2) = world_to_screen(&triangle.v1, data) else {
            return;
        };
        let Some(v3) = world_to_screen(&triangle.v2, data) else {
            return;
        };
        painter.line(vec![v1, v2, v3], Stroke::new(1.0, Color32::WHITE));
    }

    #[allow(unused)]
    fn aabb_box(&self, aabb: &Aabb, data: &Data, painter: &Painter) {
        let min = aabb.min();
        let max = aabb.max();

        let corners = [
            Vec3::new(min.x, min.y, min.z),
            Vec3::new(max.x, min.y, min.z),
            Vec3::new(min.x, max.y, min.z),
            Vec3::new(max.x, max.y, min.z),
            Vec3::new(min.x, min.y, max.z),
            Vec3::new(max.x, min.y, max.z),
            Vec3::new(min.x, max.y, max.z),
            Vec3::new(max.x, max.y, max.z),
        ];

        let screen_points: Vec<Option<Pos2>> =
            corners.iter().map(|p| world_to_screen(p, data)).collect();

        let edges = [
            (0, 1),
            (1, 3),
            (3, 2),
            (2, 0),
            (4, 5),
            (5, 7),
            (7, 6),
            (6, 4),
            (0, 4),
            (1, 5),
            (2, 6),
            (3, 7),
        ];

        for (i, j) in edges.iter() {
            if let (Some(p0), Some(p1)) = (screen_points[*i], screen_points[*j]) {
                painter.line_segment([p0, p1], Stroke::new(1.0, Color32::WHITE));
            }
        }
    }

    fn player_box(&self, painter: &Painter, player: &PlayerData, data: &Data) {
        let health_color = self.apply_alpha(self.health_color(player.health));
        let color = match &self.config.player.draw_box {
            DrawMode::None => health_color,
            DrawMode::Health => health_color,
            DrawMode::Color => {
                if player.visible {
                    self.apply_alpha(self.config.player.box_visible_color)
                } else {
                    self.apply_alpha(self.config.player.box_invisible_color)
                }
            }
        };
        let stroke = Stroke::new(self.config.hud.line_width, color);
        let icon_font = FontId::monospace(self.config.hud.font_size * 1.5);

        let midpoint = (player.position + player.head) / 2.0;
        let height = player.head.z - player.position.z + 24.0;
        let half_height = height / 2.0;
        let top = midpoint + vec3(0.0, 0.0, half_height);
        let bottom = midpoint - vec3(0.0, 0.0, half_height);

        let Some(top) = world_to_screen(&top, data) else {
            return;
        };
        let Some(bottom) = world_to_screen(&bottom, data) else {
            return;
        };
        let half_height = bottom.y - top.y;
        let width = half_height / 2.0;
        let half_width = width / 2.0;
        // quarter width
        let qw = half_width - 2.0;
        // eigth width
        let ew = qw / 2.0;

        let tl = pos2(top.x - half_width, top.y);
        let tr = pos2(top.x + half_width, top.y);
        let bl = pos2(bottom.x - half_width, bottom.y);
        let br = pos2(bottom.x + half_width, bottom.y);

        if self.config.player.draw_box != DrawMode::None {
            if self.config.player.box_mode == BoxMode::Gap {
                painter.line(
                    vec![pos2(tl.x + ew, tl.y), tl, pos2(tl.x, tl.y + qw)],
                    stroke,
                );
                painter.line(
                    vec![pos2(tr.x - ew, tl.y), tr, pos2(tr.x, tr.y + qw)],
                    stroke,
                );
                painter.line(
                    vec![pos2(bl.x + ew, bl.y), bl, pos2(bl.x, bl.y - qw)],
                    stroke,
                );
                painter.line(
                    vec![pos2(br.x - ew, bl.y), br, pos2(br.x, br.y - qw)],
                    stroke,
                );
            } else {
                painter.rect(
                    egui::Rect::from_min_max(tl, br),
                    0,
                    Color32::TRANSPARENT,
                    stroke,
                    egui::StrokeKind::Middle,
                );
            }
        }

        // health bar
        if self.config.player.health_bar {
            let x = bl.x - self.config.hud.line_width * 2.0;
            let delta = bl.y - tl.y;
            painter.line(
                vec![
                    pos2(x, bl.y),
                    pos2(x, bl.y - (delta * player.health as f32 / 100.0)),
                ],
                Stroke::new(self.config.hud.line_width, health_color),
            );
        }

        if self.config.player.armor_bar && player.armor > 0 {
            let x = bl.x
                - self.config.hud.line_width
                    * if self.config.player.health_bar {
                        4.0
                    } else {
                        2.0
                    };
            let delta = bl.y - tl.y;
            painter.line(
                vec![
                    pos2(x, bl.y),
                    pos2(x, bl.y - (delta * player.armor as f32 / 100.0)),
                ],
                Stroke::new(self.config.hud.line_width, self.apply_alpha(Color32::BLUE)),
            );
        }

        let mut offset = 0.0;
        let font_size = self.config.hud.font_size;
        let text_color = self.apply_alpha(self.config.hud.text_color);
        if self.config.player.player_name {
            self.text(
                painter,
                &player.name,
                pos2(tr.x + ew, tr.y + offset),
                Align2::LEFT_TOP,
                None,
            );
            offset += font_size;
        }

        if self.config.player.tags && player.has_defuser {
            painter.text(
                pos2(tr.x + ew, tr.y + offset),
                Align2::LEFT_TOP,
                "r",
                icon_font.clone(),
                text_color,
            );
            offset += font_size;
        }

        if self.config.player.tags && player.has_helmet {
            painter.text(
                pos2(tr.x + ew, tr.y + offset),
                Align2::LEFT_TOP,
                "q",
                icon_font.clone(),
                text_color,
            );
            offset += font_size;
        }

        if self.config.player.tags && player.has_bomb {
            painter.text(
                pos2(tr.x + ew, tr.y + offset),
                Align2::LEFT_TOP,
                "o",
                icon_font.clone(),
                text_color,
            );
        }

        if self.config.player.weapon_icon {
            painter.text(
                pos2(bl.x + half_width, bl.y),
                Align2::CENTER_TOP,
                player.weapon.to_icon(),
                icon_font.clone(),
                text_color,
            );
        }
    }

    fn skeleton(&self, painter: &Painter, player: &PlayerData, data: &Data) {
        let color = match &self.config.player.draw_skeleton {
            DrawMode::None => return,
            DrawMode::Health => self.apply_alpha(self.health_color(player.health)),
            DrawMode::Color => self.apply_alpha(self.config.player.skeleton_color),
        };
        let stroke = Stroke::new(self.config.hud.line_width, color);

        for (a, b) in &Bones::CONNECTIONS {
            let a = player.bones.get(a).unwrap();
            let b = player.bones.get(b).unwrap();

            let Some(a) = world_to_screen(a, data) else {
                continue;
            };
            let Some(b) = world_to_screen(b, data) else {
                continue;
            };

            painter.line(vec![a, b], stroke);
        }

        // head circle
        if !self.config.player.head_circle {
            return;
        }
        let neck = player.bones.get(&Bones::Neck).unwrap();
        let spine = player.bones.get(&Bones::Spine3).unwrap();

        let Some(neck) = world_to_screen(neck, data) else {
            return;
        };
        let Some(spine) = world_to_screen(spine, data) else {
            return;
        };

        let height = spine.y - neck.y;
        let pos = pos2(neck.x - (spine.x - neck.x) / 2.0, neck.y - height / 2.0);
        painter.circle_stroke(pos, height / 2.0, stroke);
    }

    fn health_color(&self, health: i32) -> Color32 {
        let health = health.clamp(0, 100);

        let (r, g) = if health <= 50 {
            let factor = health as f32 / 50.0;
            (255, (255.0 * factor) as u8)
        } else {
            let factor = 1.0 - (health - 50) as f32 / 50.0;
            ((255.0 * factor) as u8, 255)
        };

        Color32::from_rgb(r, g, 0)
    }

    fn text(
        &self,
        painter: &Painter,
        text: impl AsRef<str>,
        position: Pos2,
        align: Align2,
        color: Option<Color32>,
    ) {
        let font = FontId::proportional(self.config.hud.font_size);
        let color = match color {
            Some(color) => color,
            None => self.apply_alpha(self.config.hud.text_color),
        };
        if self.config.hud.text_outline {
            for (pos, color) in outline(position, color) {
                painter.text(pos, align, text.as_ref(), font.clone(), color);
            }
        } else {
            painter.text(position, align, text.as_ref(), font, color);
        }
    }

    pub fn render(&mut self) {
        use glow::HasContext as _;

        let self_ptr = self as *mut Self;
        self.gui_window.as_mut().unwrap().make_current().unwrap();
        self.gui_glow
            .as_mut()
            .unwrap()
            .run(self.gui_window.as_mut().unwrap().window(), |ctx| {
                (unsafe { &mut *self_ptr }).gui(ctx)
            });

        unsafe {
            self.gui_gl
                .as_mut()
                .unwrap()
                .clear_color(0.0, 0.0, 0.0, 1.0);
            self.gui_gl.as_mut().unwrap().clear(glow::COLOR_BUFFER_BIT);
        }

        self.gui_glow
            .as_mut()
            .unwrap()
            .paint(self.gui_window.as_mut().unwrap().window());

        self.gui_window.as_mut().unwrap().swap_buffers().unwrap();

        self.overlay_window
            .as_mut()
            .unwrap()
            .make_current()
            .unwrap();
        self.overlay_glow.as_mut().unwrap().run(
            self.overlay_window.as_mut().unwrap().window(),
            move |egui_ctx| {
                (unsafe { &mut *self_ptr }).overlay(egui_ctx);
            },
        );

        unsafe {
            self.overlay_gl
                .as_mut()
                .unwrap()
                .clear_color(0.0, 0.0, 0.0, 0.0);
            self.overlay_gl
                .as_mut()
                .unwrap()
                .clear(glow::COLOR_BUFFER_BIT);
        }

        self.overlay_glow
            .as_mut()
            .unwrap()
            .paint(self.overlay_window.as_mut().unwrap().window());

        self.overlay_window
            .as_mut()
            .unwrap()
            .swap_buffers()
            .unwrap();
    }
}

fn collapsing_open(ui: &mut Ui, title: &str, add_body: impl FnOnce(&mut Ui)) {
    CollapsingHeader::new(title)
        .default_open(true)
        .show(ui, add_body);
}

fn find_player(steam_id: u64, data: &Data) -> Option<&PlayerData> {
    if data.local_player.steam_id == steam_id {
        return Some(&data.local_player);
    }
    data.players
        .iter()
        .find(|p| p.steam_id == steam_id)
        .or(data.friendlies.iter().find(|p| p.steam_id == steam_id))
}
