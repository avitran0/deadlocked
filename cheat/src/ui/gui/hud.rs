use egui::{DragValue, Ui};

use crate::ui::{
    app::AppState,
    gui::helpers::{
        checkbox, collapsing_open, color_picker, combo_box, drag, scroll, text_settings_button,
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
        });
    }
}
