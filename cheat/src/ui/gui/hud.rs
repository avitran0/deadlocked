use egui::{DragValue, Ui};

use crate::ui::{
    app::AppState,
    gui::helpers::{
        checkbox, checkbox_hover, collapsing_open, color_picker, combo_box, drag, reset_button,
        scroll, text_settings_button,
    },
};

impl AppState {
    pub fn hud_settings(&mut self, ui: &mut Ui) {
        scroll(ui, "hud", |ui| {
            ui.columns(2, |cols| {
                let left = &mut cols[0];
                self.hud_left(left);
                let right = &mut cols[1];
                self.hud_right(right);
            });

            collapsing_open(ui, "Colors", |ui| {
                if color_picker(
                    ui,
                    "Crosshair Color",
                    &mut self.config.hud.sniper_crosshair.color,
                ) {
                    self.send_config_game();
                }

                if reset_button(ui) {
                    self.config.hud.sniper_crosshair.color =
                        crate::config::hud::CrosshairConfig::default().color;
                    self.send_config_game();
                }
            });

            ui.collapsing("Grenade Trails", |ui| {
                if checkbox(
                    ui,
                    "Enable Grenade Trails",
                    &mut self.config.hud.grenade_trails.enabled,
                ) {
                    self.send_config_game();
                }

                if checkbox(
                    ui,
                    "Inferno Polygon",
                    &mut self.config.hud.grenade_trails.inferno_poly,
                ) {
                    self.send_config_game();
                }

                if color_picker(
                    ui,
                    "Smoke Trail Color",
                    &mut self.config.hud.grenade_trails.smoke,
                ) {
                    self.send_config_game();
                }

                if color_picker(
                    ui,
                    "Molotov Trail Color",
                    &mut self.config.hud.grenade_trails.molotov,
                ) {
                    self.send_config_game();
                }

                if color_picker(
                    ui,
                    "Incendiary Trail Color",
                    &mut self.config.hud.grenade_trails.incendiary,
                ) {
                    self.send_config_game();
                }

                if color_picker(
                    ui,
                    "Flash Trail Color",
                    &mut self.config.hud.grenade_trails.flash,
                ) {
                    self.send_config_game();
                }

                if color_picker(
                    ui,
                    "HE Grenade Trail Color",
                    &mut self.config.hud.grenade_trails.he,
                ) {
                    self.send_config_game();
                }

                if color_picker(
                    ui,
                    "Decoy Trail Color",
                    &mut self.config.hud.grenade_trails.decoy,
                ) {
                    self.send_config_game();
                }

                if reset_button(ui) {
                    self.config.hud.grenade_trails = crate::config::hud::TrailConfig::default();
                    self.send_config_game();
                }
            });
        });
    }

    fn hud_left(&mut self, ui: &mut Ui) {
        collapsing_open(ui, "HUD", |ui| {
            ui.horizontal(|ui| {
                if checkbox(ui, "Bomb Timer", &mut self.config.hud.bomb_timer) {
                    self.send_config_game();
                }
                text_settings_button(ui, &mut self.text_popup, "bomb_timer");
            });

            if checkbox(ui, "FOV Circle", &mut self.config.hud.fov_circle) {
                self.send_config_game();
            }

            ui.horizontal(|ui| {
                if checkbox(ui, "Dropped Weapons", &mut self.config.hud.dropped_weapons) {
                    self.send_config_game();
                }
                text_settings_button(ui, &mut self.text_popup, "weapon_name");
            });

            ui.horizontal(|ui| {
                if checkbox(ui, "Keybind List", &mut self.config.hud.keybind_list) {
                    self.send_config_game();
                }
                text_settings_button(ui, &mut self.text_popup, "keybind_list");
            });

            ui.horizontal(|ui| {
                if checkbox(ui, "Spectator List", &mut self.config.hud.spectator_list) {
                    self.send_config_game();
                }
                text_settings_button(ui, &mut self.text_popup, "spectator_list");
            });

            if reset_button(ui) {
                let defaults = crate::config::hud::HudConfig::default();
                self.config.hud.bomb_timer = defaults.bomb_timer;
                self.config.hud.fov_circle = defaults.fov_circle;
                self.config.hud.dropped_weapons = defaults.dropped_weapons;
                self.config.hud.keybind_list = defaults.keybind_list;
                self.config.hud.spectator_list = defaults.spectator_list;
                self.send_config_game();
            }
        });

        ui.collapsing("Minimap", |ui| {
            if checkbox_hover(
                ui,
                "Enabled",
                "In-game radar overlay (not the web radar)",
                &mut self.config.hud.minimap.enabled,
            ) {
                self.send_config_game();
            }

            if drag(
                ui,
                "Size",
                DragValue::new(&mut self.config.hud.minimap.size)
                    .range(80.0..=420.0)
                    .max_decimals(0)
                    .speed(1.0),
            ) {
                self.send_config_game();
            }

            if drag(
                ui,
                "Range",
                DragValue::new(&mut self.config.hud.minimap.range)
                    .range(400.0..=8000.0)
                    .max_decimals(0)
                    .speed(25.0),
            ) {
                self.send_config_game();
            }

            if drag(
                ui,
                "Opacity",
                DragValue::new(&mut self.config.hud.minimap.opacity)
                    .range(0.2..=1.0)
                    .max_decimals(2)
                    .speed(0.01),
            ) {
                self.send_config_game();
            }

            if drag(
                ui,
                "Icon Size",
                DragValue::new(&mut self.config.hud.minimap.icon_size)
                    .range(3.0..=16.0)
                    .max_decimals(1)
                    .speed(0.1),
            ) {
                self.send_config_game();
            }

            if combo_box(
                ui,
                "minimap_pos",
                "Position",
                &mut self.config.hud.minimap.position,
            ) {
                self.send_config_game();
            }

            if checkbox_hover(
                ui,
                "Rotate With View",
                "Keep your facing direction at the top of the radar",
                &mut self.config.hud.minimap.rotate_with_view,
            ) {
                self.send_config_game();
            }

            if checkbox(
                ui,
                "Show Names",
                &mut self.config.hud.minimap.show_names,
            ) {
                self.send_config_game();
            }

            if checkbox(
                ui,
                "Show Bomb",
                &mut self.config.hud.minimap.show_bomb,
            ) {
                self.send_config_game();
            }

            if checkbox(
                ui,
                "Show Teammates",
                &mut self.config.hud.minimap.show_teammates,
            ) {
                self.send_config_game();
            }

            if checkbox(
                ui,
                "Show Smoke",
                &mut self.config.hud.minimap.show_smoke,
            ) {
                self.send_config_game();
            }

            if checkbox(
                ui,
                "Show Molotov",
                &mut self.config.hud.minimap.show_molotov,
            ) {
                self.send_config_game();
            }

            if checkbox(
                ui,
                "Range Rings",
                &mut self.config.hud.minimap.show_rings,
            ) {
                self.send_config_game();
            }

            if checkbox_hover(
                ui,
                "Clamp To Edge",
                "Players outside range stay on the radar rim",
                &mut self.config.hud.minimap.clamp_to_edge,
            ) {
                self.send_config_game();
            }

            if color_picker(
                ui,
                "Background",
                &mut self.config.hud.minimap.background,
            ) {
                self.send_config_game();
            }

            if color_picker(
                ui,
                "Enemy Color",
                &mut self.config.hud.minimap.enemy_color,
            ) {
                self.send_config_game();
            }

            if color_picker(
                ui,
                "Teammate Color",
                &mut self.config.hud.minimap.teammate_color,
            ) {
                self.send_config_game();
            }

            if color_picker(
                ui,
                "Bomb Color",
                &mut self.config.hud.minimap.bomb_color,
            ) {
                self.send_config_game();
            }

            if color_picker(
                ui,
                "Local Color",
                &mut self.config.hud.minimap.local_color,
            ) {
                self.send_config_game();
            }

            if color_picker(
                ui,
                "Smoke Color",
                &mut self.config.hud.minimap.smoke_color,
            ) {
                self.send_config_game();
            }

            if color_picker(
                ui,
                "Molotov Color",
                &mut self.config.hud.minimap.molotov_color,
            ) {
                self.send_config_game();
            }

            if reset_button(ui) {
                self.config.hud.minimap = crate::config::hud::MinimapConfig::default();
                self.send_config_game();
            }
        });

        ui.collapsing("Sniper Crosshair", |ui| {
            if checkbox(ui, "Enabled", &mut self.config.hud.sniper_crosshair.enabled) {
                self.send_config_game();
            }

            if drag(
                ui,
                "Line Length",
                DragValue::new(&mut self.config.hud.sniper_crosshair.line_length)
                    .range(0.1..=500.0)
                    .max_decimals(1)
                    .speed(0.2),
            ) {
                self.send_config_game();
            }

            if drag(
                ui,
                "Line Width",
                DragValue::new(&mut self.config.hud.sniper_crosshair.line_width)
                    .range(0.1..=10.0)
                    .max_decimals(1)
                    .speed(0.005),
            ) {
                self.send_config_game();
            }

            if drag(
                ui,
                "Gap",
                DragValue::new(&mut self.config.hud.sniper_crosshair.gap)
                    .range(0.0..=200.0)
                    .max_decimals(1)
                    .speed(0.2),
            ) {
                self.send_config_game();
            }

            if reset_button(ui) {
                self.config.hud.sniper_crosshair =
                    crate::config::hud::CrosshairConfig::default();
                self.send_config_game();
            }
        });
    }

    fn hud_right(&mut self, ui: &mut Ui) {
        collapsing_open(ui, "Appearance", |ui| {
            if checkbox(ui, "Text Outline", &mut self.config.hud.text_outline) {
                self.send_config_game();
            }

            if drag(
                ui,
                "Line Width",
                DragValue::new(&mut self.config.hud.line_width)
                    .range(0.1..=8.0)
                    .speed(0.02)
                    .max_decimals(1),
            ) {
                self.send_config_game();
            }

            if combo_box(ui, "font", "Font", &mut self.config.font) {
                self.config.font.set(ui.ctx());
                if let Some(ctx) = &self.overlay_egui {
                    self.config.font.set(ctx);
                }
                self.send_config_game();
            }

            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Status Text");
                text_settings_button(ui, &mut self.text_popup, "status_text");
            });

            ui.horizontal(|ui| {
                ui.label("Grenade Name");
                text_settings_button(ui, &mut self.text_popup, "grenade_name");
            });

            ui.horizontal(|ui| {
                ui.label("Grenade Lineup");
                text_settings_button(ui, &mut self.text_popup, "grenade_lineup");
            });

            if reset_button(ui) {
                let defaults = crate::config::hud::HudConfig::default();
                self.config.hud.text_outline = defaults.text_outline;
                self.config.hud.line_width = defaults.line_width;
                self.config.font = crate::config::Config::default().font;
                self.config.font.set(ui.ctx());
                if let Some(ctx) = &self.overlay_egui {
                    self.config.font.set(ctx);
                }
                self.send_config_game();
            }
        });

        ui.collapsing("Advanced", |ui| {
            if checkbox(ui, "Debug Overlay", &mut self.config.hud.debug) {
                self.send_config_game();
            }

            if drag(
                ui,
                "FPS",
                DragValue::new(&mut self.config.fps).range(30..=500),
            ) {
                self.send_config_game();
            }

            if reset_button(ui) {
                self.config.hud.debug = false;
                self.config.fps = crate::config::Config::default().fps;
                self.send_config_game();
            }
        });
    }
}
