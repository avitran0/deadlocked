use egui::{Align, Button, CollapsingHeader, Context, DragValue, Ui};
use egui_glow::glow;
use strum::IntoEnumIterator;

use crate::{
    config::{
        CONFIG_PATH, Config, TargetingMode, TriggerbotMode, VERSION, WeaponConfig,
        available_configs, delete_config, parse_config, write_config,
    },
    constants::cs2::GRENADES,
    cs2::{bones::Bones, entity::weapon::Weapon},
    drag_range::DragRange,
    key_codes::KeyCode,
    message::{Envelope, GameStatus, Message, RadarStatus, Target},
    os::mouse::{DeviceStatus, discover_mice},
    ui::{
        app::App,
        color::Colors,
        grenades::{Grenade, write_grenades},
    },
};

#[cfg(any(feature = "unsafe", feature = "visuals"))]
use egui::Color32;

#[derive(PartialEq)]
pub enum Tab {
    Aimbot,
    #[cfg(feature = "visuals")]
    Player,
    #[cfg(feature = "visuals")]
    Hud,
    Radar,
    Grenades,
    #[cfg(feature = "unsafe")]
    Unsafe,
    Config,
}

#[derive(PartialEq)]
pub enum AimbotTab {
    Global,
    Weapon,
}

impl App {
    pub fn send_config(&self) {
        self.send_message(Message::Config(Box::new(self.config.clone())), Target::Game);
        self.save();
    }

    pub fn send_message(&self, message: Message, target: Target) {
        if self.tx.send(Envelope { target, message }).is_err() {
            std::process::exit(1);
        }
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
                #[cfg(feature = "visuals")]
                {
                    ui.selectable_value(&mut self.current_tab, Tab::Player, "\u{f0013} Player");
                    ui.selectable_value(&mut self.current_tab, Tab::Hud, "\u{f0379} Hud");
                }
                ui.selectable_value(&mut self.current_tab, Tab::Radar, "\u{f0437} Radar");
                ui.selectable_value(&mut self.current_tab, Tab::Grenades, "\u{f0691} Grenades");
                #[cfg(feature = "unsafe")]
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
                #[cfg(feature = "visuals")]
                Tab::Player => self.player_settings(ui),
                #[cfg(feature = "visuals")]
                Tab::Hud => self.hud_settings(ui),
                Tab::Radar => self.radar_settings(ui),
                Tab::Grenades => self.grenade_settings(ui),
                #[cfg(feature = "unsafe")]
                Tab::Unsafe => self.unsafe_settings(ui),
                Tab::Config => self.config_settings(ui, ctx),
            }
        });
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

            if ui
                .checkbox(
                    &mut self.weapon_config().aimbot.target_friendlies,
                    "Target Friendlies",
                )
                .on_hover_text("Only active in custom game modes (workshop/custom maps)")
                .changed()
            {
                self.send_config();
            }

            if ui
                .checkbox(
                    &mut self.weapon_config().aimbot.distance_adjusted_fov,
                    "Distance-Adjusted FOV",
                )
                .on_hover_text("Adjusts FOV based on target distance")
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
                            .range(0.0..=20.0)
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

            egui::ComboBox::new("targeting_mode", "Targeting Mode")
                .selected_text(format!("{:?}", self.weapon_config().aimbot.targeting_mode))
                .show_ui(ui, |ui| {
                    for mode in TargetingMode::iter() {
                        let text = format!("{:?}", &mode);
                        if ui
                            .selectable_value(
                                &mut self.weapon_config().aimbot.targeting_mode,
                                mode,
                                text,
                            )
                            .clicked()
                        {
                            self.send_config();
                        }
                    }
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
                        DragValue::new(&mut self.weapon_config().triggerbot.additional_duration_ms)
                            .range(0..=2000)
                            .speed(10.0),
                    )
                    .changed()
                {
                    self.send_config();
                }
                ui.label("Additional Duration (ms)");
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

    #[cfg(feature = "visuals")]
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

    #[cfg(feature = "visuals")]
    fn player_left(&mut self, ui: &mut Ui) {
        collapsing_open(ui, "Players", |ui| {
            if ui
                .checkbox(&mut self.config.player.enabled, "Enable")
                .changed()
            {
                self.send_config();
            }

            egui::ComboBox::new("esp_hotkey", "ESP Hotkey")
                .selected_text(format!("{:?}", self.config.player.esp_hotkey))
                .show_ui(ui, |ui| {
                    for key_code in KeyCode::iter() {
                        let text = format!("{:?}", &key_code);
                        if ui
                            .selectable_value(&mut self.config.player.esp_hotkey, key_code, text)
                            .clicked()
                        {
                            self.send_config();
                        }
                    }
                });

            if ui
                .checkbox(&mut self.config.player.show_friendlies, "Show Friendlies")
                .on_hover_text("Only active in custom game modes (workshop/custom maps)")
                .changed()
            {
                self.send_config();
            }

            egui::ComboBox::new("draw_box", "Box")
                .selected_text(format!("{:?}", self.config.player.draw_box))
                .show_ui(ui, |ui| {
                    use crate::config::DrawMode;

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
                    use crate::config::BoxMode;

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
                    use crate::config::DrawMode;

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

            if ui
                .checkbox(&mut self.config.player.head_circle, "Head Circle")
                .changed()
            {
                self.send_config();
            }
        });
    }

    #[cfg(feature = "visuals")]
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

    #[cfg(feature = "visuals")]
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

                collapsing_open(ui, "Grenade Trails", |ui| {
                    if ui
                    .checkbox(&mut self.config.hud.grenade_trails, "Grenade Trails")
                    .changed()
                    {
                        self.send_config();
                    }

                    if let Some(color) =
                    self.color_picker(ui, &self.config.hud.smoke_trail_color, "Smoke Trail Color")
                    {
                        self.config.hud.smoke_trail_color = color;
                        self.send_config();
                    }

                    if let Some(color) = self.color_picker(
                        ui,
                        &self.config.hud.molotov_trail_color,
                        "Molotov Trail Color",
                    ) {
                        self.config.hud.molotov_trail_color = color;
                        self.send_config();
                    }

                    if let Some(color) = self.color_picker(
                        ui,
                        &self.config.hud.incendiary_trail_color,
                        "Incendiary Trail Color",
                    ) {
                        self.config.hud.incendiary_trail_color = color;
                        self.send_config();
                    }

                    if let Some(color) =
                    self.color_picker(ui, &self.config.hud.flash_trail_color, "Flash Trail Color")
                    {
                        self.config.hud.flash_trail_color = color;
                        self.send_config();
                    }

                    if let Some(color) = self.color_picker(
                        ui,
                        &self.config.hud.he_trail_color,
                        "HE Grenade Trail Color",
                    ) {
                        self.config.hud.he_trail_color = color;
                        self.send_config();
                    }

                    if let Some(color) =
                    self.color_picker(ui, &self.config.hud.decoy_trail_color, "Decoy Trail Color")
                    {
                        self.config.hud.decoy_trail_color = color;
                        self.send_config();
                    }
                });
            });
    }

    #[cfg(feature = "visuals")]
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
                .checkbox(&mut self.config.hud.spectators, "Spectator List")
                .changed()
            {
                self.send_config();
            }
        });
    }

    #[cfg(feature = "visuals")]
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

    fn radar_settings(&mut self, ui: &mut Ui) {
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
                        ui.horizontal(|ui| {
                            if ui.button("Open").clicked() {
                                let link =
                                    format!("http://{}/?uuid={}", self.config.radar.url, uuid);
                                std::process::Command::new("xdg-open")
                                    .arg(&link)
                                    .status()
                                    .unwrap();
                                log::info!("opened link ({link})");
                            }

                            if ui.button("Copy Link").clicked() {
                                let link =
                                    format!("http://{}/?uuid={}", self.config.radar.url, uuid);
                                log::info!("copied link ({link})");
                                // ctx.copy_text(link);
                                self.clipboard.set_text(link).unwrap();
                            }
                        });
                    }
                });
            });
    }

    fn grenade_settings(&mut self, ui: &mut Ui) {
        if self.current_grenade.is_some() {
            self.edit_grenade(ui);
        } else {
            self.record_grenade(ui);
        }

        // grenade list
        ui.collapsing("Grenade List", |ui| {
            self.grenade_list(ui);
        });
    }

    fn grenade_list(&mut self, ui: &mut Ui) {
        let mut should_write = false;

        for (map, grenades) in &mut self.grenades {
            let mut delete_grenade_index = None;

            ui.collapsing(map, |ui| {
                for (index, grenade) in grenades.iter().enumerate() {
                    let active = match &self.current_grenade {
                        Some(grenade) => &grenade.0 == map && grenade.1 == index,
                        None => false,
                    };
                    ui.horizontal(|ui| {
                        if ui.selectable_label(active, &grenade.name).clicked() {
                            self.current_grenade = match self.current_grenade {
                                Some((ref g_map, ref g_index))
                                    if g_map == map && *g_index == index =>
                                {
                                    None
                                }
                                _ => Some((map.to_owned(), index)),
                            };
                        }
                        if ui.button("\u{f0a7a}").clicked() {
                            delete_grenade_index = Some(index);
                        }
                    });
                }
                if let Some(index) = delete_grenade_index {
                    grenades.remove(index);
                    should_write = true;
                }
            });
        }

        if should_write {
            write_grenades(&self.grenades);
        }
    }

    fn record_grenade(&mut self, ui: &mut Ui) {
        collapsing_open(ui, "Record Grenade", |ui| {
            let data = self.data.lock().unwrap();

            if !data.in_game {
                ui.label("Not in game.");
                return;
            }

            let grenade = if !GRENADES.contains(&data.local_player.weapon) {
                ui.colored_label(Colors::YELLOW, "Invalid Weapon");
                None
            } else {
                Some(data.local_player.weapon.clone())
            };

            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut self.new_grenade.name);
                ui.label("Name");
            });

            ui.horizontal(|ui| {
                ui.text_edit_multiline(&mut self.new_grenade.description);
                ui.label("Description");
            });

            ui.checkbox(&mut self.new_grenade.modifiers.jump, "Jump");
            ui.checkbox(&mut self.new_grenade.modifiers.duck, "Duck");
            ui.checkbox(&mut self.new_grenade.modifiers.run, "Run");

            if ui.button("Record").clicked() {
                let grenade = match grenade {
                    Some(grenade) => grenade,
                    None => {
                        log::info!("no grenade in hand");
                        return;
                    }
                };

                let map = &data.map_name;
                let grenade_list = match self.grenades.get_mut(map) {
                    Some(list) => list,
                    None => {
                        self.grenades.insert(map.to_owned(), Vec::new());
                        self.grenades.get_mut(map).unwrap()
                    }
                };

                let mut new_grenade = Grenade::new();
                std::mem::swap(&mut new_grenade, &mut self.new_grenade);

                new_grenade.weapon = grenade;
                new_grenade.position = data.local_player.position;
                new_grenade.view_angles = data.view_angles;

                grenade_list.push(new_grenade);
                write_grenades(&self.grenades);
            }
        });
    }

    fn edit_grenade(&mut self, ui: &mut Ui) {
        collapsing_open(ui, "Edit Grenade", |ui| {
            let (map, index) = match &self.current_grenade {
                Some(grenade) => grenade,
                None => return,
            };

            let grenade = &mut self.grenades.get_mut(map).unwrap()[*index];

            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut grenade.name);
                ui.label("Name");
            });

            ui.horizontal(|ui| {
                ui.text_edit_multiline(&mut grenade.description);
                ui.label("Description");
            });

            ui.checkbox(&mut grenade.modifiers.jump, "Jump");
            ui.checkbox(&mut grenade.modifiers.duck, "Duck");
            ui.checkbox(&mut grenade.modifiers.run, "Run");
        });
    }

    #[cfg(feature = "unsafe")]
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

    #[cfg(feature = "unsafe")]
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

    #[cfg(feature = "unsafe")]
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
                    self.config.misc.desired_fov = crate::constants::cs2::DEFAULT_FOV;
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
                            self.send_config();
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
            ui.ctx()
                .style_mut(|style| style.visuals.selection.bg_fill = self.config.accent_color);
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

    #[cfg(any(feature = "unsafe", feature = "visuals"))]
    fn color_picker(&self, ui: &mut Ui, color: &Color32, label: &str) -> Option<Color32> {
        let [mut r, mut g, mut b, _] = color.to_array();
        let res = ui
            .horizontal(|ui| {
                let (response, painter) =
                    ui.allocate_painter(ui.spacing().interact_size, egui::Sense::hover());
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

    pub fn render(&mut self) {
        use glow::HasContext as _;

        let self_ptr = self as *mut Self;

        let gui_window = self.gui_window.as_ref().unwrap();
        let gui_glow = self.gui_glow.as_mut().unwrap();

        gui_window.make_current().unwrap();
        gui_glow.run(gui_window.window(), |ctx| {
            (unsafe { &mut *self_ptr }).gui(ctx)
        });

        unsafe {
            let gui_gl = self.gui_gl.as_ref().unwrap();
            gui_gl.clear_color(0.0, 0.0, 0.0, 1.0);
            gui_gl.clear(glow::COLOR_BUFFER_BIT);
        }

        gui_glow.paint(gui_window.window());

        gui_window.swap_buffers().unwrap();

        #[cfg(feature = "visuals")]
        {
            let overlay_window = self.overlay_window.as_ref().unwrap();
            let overlay_glow = self.overlay_glow.as_mut().unwrap();

            overlay_window.window().set_cursor_hittest(false).unwrap();
            overlay_window.make_current().unwrap();

            overlay_glow.run(overlay_window.window(), move |egui_ctx| {
                (unsafe { &mut *self_ptr }).overlay(egui_ctx);
            });

            unsafe {
                let overlay_gl = self.overlay_gl.as_ref().unwrap();
                overlay_gl.clear_color(0.0, 0.0, 0.0, 0.0);
                overlay_gl.clear(glow::COLOR_BUFFER_BIT);
            }

            overlay_glow.paint(overlay_window.window());

            overlay_window.swap_buffers().unwrap();
        }
    }
}

fn collapsing_open(ui: &mut Ui, title: &str, add_body: impl FnOnce(&mut Ui)) {
    CollapsingHeader::new(title)
        .default_open(true)
        .show(ui, add_body);
}
