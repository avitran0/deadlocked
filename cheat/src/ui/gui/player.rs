use egui::{DragValue, Ui};

use crate::{
    config::player::ColorValues,
    ui::{
        app::AppState,
        gui::helpers::{
            checkbox, checkbox_hover, collapsing_open, color_picker, combo_box, drag, keybind,
            scroll, text_settings_button,
        },
    },
};

/// every widget for one `ColorValues` block, reused for the shared/global
/// colors and each of Box/Skeleton/Model/Hitbox ESP's own override - so
/// none of these fields get defined or drawn more than once
fn color_values_editor(ui: &mut Ui, colors: &mut ColorValues) -> bool {
    let mut changed = false;
    changed |= color_picker(ui, "Enemy", &mut colors.enemy_color);
    changed |= color_picker(ui, "Ally", &mut colors.ally_color);
    changed |= color_picker(ui, "Visible", &mut colors.visible_color);
    changed |= color_picker(ui, "Hidden", &mut colors.hidden_color);
    changed |= color_picker(ui, "Distance Near", &mut colors.distance_near_color);
    changed |= color_picker(ui, "Distance Far", &mut colors.distance_far_color);
    changed |= drag(
        ui,
        "Distance Max",
        DragValue::new(&mut colors.distance_max)
            .range(100.0..=8000.0)
            .speed(10.0),
    );
    changed |= checkbox_hover(
        ui,
        "Gold C4 Carrier",
        "Overrides whatever color the mode above would use for the bomb carrier, while they're visible.",
        &mut colors.c4_carrier_highlight,
    );
    changed |= color_picker(ui, "C4 Carrier", &mut colors.c4_carrier_color);
    changed
}

impl AppState {
    pub fn player_settings(&mut self, ui: &mut Ui) {
        scroll(ui, "player", |ui| {
            ui.columns(2, |cols| {
                let left = &mut cols[0];
                self.player_left(left);
                let right = &mut cols[1];
                self.player_right(right);
            });

            collapsing_open(ui, "Colors", |ui| {
                ui.label(
                    "Shared/global - used by any ESP below that hasn't enabled its own override:",
                );
                if color_values_editor(ui, &mut self.config.player.colors) {
                    self.send_config_game();
                }

                ui.collapsing("Advanced: Box Colors", |ui| {
                    if checkbox_hover(
                        ui,
                        "Override",
                        "Use Box ESP's own colors below instead of the shared ones above.",
                        &mut self.config.player.box_colors_override,
                    ) {
                        self.send_config_game();
                    }
                    if color_values_editor(ui, &mut self.config.player.box_colors) {
                        self.send_config_game();
                    }
                });

                ui.collapsing("Advanced: Skeleton Colors", |ui| {
                    if checkbox_hover(
                        ui,
                        "Override",
                        "Use Skeleton ESP's own colors below instead of the shared ones above.",
                        &mut self.config.player.skeleton_colors_override,
                    ) {
                        self.send_config_game();
                    }
                    if color_values_editor(ui, &mut self.config.player.skeleton_colors) {
                        self.send_config_game();
                    }
                });

                ui.collapsing("Advanced: Model ESP Colors", |ui| {
                    if combo_box(
                        ui,
                        "model_color_mode",
                        "Color Mode",
                        &mut self.config.player.model_color_mode,
                    ) {
                        self.send_config_game();
                    }
                    if checkbox_hover(
                        ui,
                        "Override",
                        "Use Model ESP's own colors below instead of the shared ones above.",
                        &mut self.config.player.model_colors_override,
                    ) {
                        self.send_config_game();
                    }
                    if color_values_editor(ui, &mut self.config.player.model_colors) {
                        self.send_config_game();
                    }
                    if color_picker(ui, "Outline", &mut self.config.player.model_outline_color) {
                        self.send_config_game();
                    }
                    ui.label("Outline alpha 0 = off; raise it to tint the mesh's silhouette edge toward this color.");
                });

                ui.collapsing("Advanced: Hitbox ESP Colors", |ui| {
                    if combo_box(
                        ui,
                        "hitbox_color_mode",
                        "Color Mode",
                        &mut self.config.player.hitbox_color_mode,
                    ) {
                        self.send_config_game();
                    }
                    if checkbox_hover(
                        ui,
                        "Override",
                        "Use Hitbox ESP's own colors below instead of the shared ones above.",
                        &mut self.config.player.hitbox_colors_override,
                    ) {
                        self.send_config_game();
                    }
                    if color_values_editor(ui, &mut self.config.player.hitbox_colors) {
                        self.send_config_game();
                    }
                });
            });
        });
    }

    fn player_left(&mut self, ui: &mut Ui) {
        collapsing_open(ui, "Players", |ui| {
            if checkbox(ui, "Player", &mut self.config.player.enabled) {
                self.send_config_game();
            }

            if checkbox(ui, "Chicken", &mut self.config.player.chicken) {
                self.send_config_game();
            }

            if keybind(
                ui,
                "esp_hotkey",
                "ESP Hotkey",
                &mut self.config.player.esp_hotkey,
            ) {
                self.send_config_game();
            }

            if checkbox(
                ui,
                "Show Friendlies",
                &mut self.config.player.show_friendlies,
            ) {
                self.send_config_game();
            }

            if combo_box(ui, "draw_box", "Box", &mut self.config.player.draw_box) {
                self.send_config_game();
            }

            if combo_box(ui, "box_mode", "Box Mode", &mut self.config.player.box_mode) {
                self.send_config_game();
            }

            if combo_box(
                ui,
                "draw_skeleton",
                "Skeleton",
                &mut self.config.player.draw_skeleton,
            ) {
                self.send_config_game();
            }

            if combo_box(
                ui,
                "draw_model",
                "Model",
                &mut self.config.player.draw_model,
            ) {
                self.send_config_game();
            }

            if combo_box(
                ui,
                "model_part_visibility",
                "Model Part Visibility",
                &mut self.config.player.model_part_visibility,
            ) {
                self.send_config_game();
            }

            if combo_box(
                ui,
                "hitbox_esp",
                "Hitbox ESP",
                &mut self.config.player.hitbox_esp,
            ) {
                self.send_config_game();
            }

            if checkbox(ui, "Head Circle", &mut self.config.player.head_circle) {
                self.send_config_game();
            }

            if checkbox_hover(
                ui,
                "Visible Only",
                "Only show visible players",
                &mut self.config.player.visible_only,
            ) {
                self.send_config_game();
            }
        });
    }

    fn player_right(&mut self, ui: &mut Ui) {
        collapsing_open(ui, "Info", |ui| {
            if ui
                .checkbox(&mut self.config.player.health_bar, "Health Bar")
                .changed()
            {
                self.send_config_game();
            }

            if ui
                .checkbox(&mut self.config.player.armor_bar, "Armor Bar")
                .changed()
            {
                self.send_config_game();
            }

            ui.horizontal(|ui| {
                if ui
                    .checkbox(&mut self.config.player.player_name, "Player Name")
                    .changed()
                {
                    self.send_config_game();
                }
                text_settings_button(ui, &mut self.text_popup, "player_name");
            });

            ui.horizontal(|ui| {
                if ui
                    .checkbox(&mut self.config.player.weapon_icon, "Weapon Icon")
                    .changed()
                {
                    self.send_config_game();
                }
                text_settings_button(ui, &mut self.text_popup, "weapon_icon");
            });

            ui.horizontal(|ui| {
                ui.label("Ammo");
                text_settings_button(ui, &mut self.text_popup, "ammo_text");
            });

            ui.horizontal(|ui| {
                if ui
                    .checkbox(&mut self.config.player.tags, "Show Tags")
                    .changed()
                {
                    self.send_config_game();
                }
                text_settings_button(ui, &mut self.text_popup, "player_tags");
            });
        });

        ui.collapsing("Sound ESP", |ui| {
            if checkbox_hover(
                ui,
                "Enabled",
                "Show a circle under players when they make sound",
                &mut self.config.player.sound.enabled,
            ) {
                self.send_config_game();
            }

            if drag(
                ui,
                "Fadeout Time (s)",
                DragValue::new(&mut self.config.player.sound.fadeout_duration)
                    .range(0.0..=10.0)
                    .speed(0.01),
            ) {
                self.send_config_game();
            }

            if checkbox(
                ui,
                "Show Visible",
                &mut self.config.player.sound.show_visible,
            ) {
                self.send_config_game();
            }

            ui.collapsing("Ranges", |ui| {
                ui.horizontal(|ui| {
                    let response = ui.add(
                        egui::DragValue::new(&mut self.config.player.sound.footstep_diameter)
                            .speed(10.0)
                            .range(200.0..=6000.0),
                    );

                    ui.label("Footstep");

                    if ui.button("↺").on_hover_text("Reset").clicked() {
                        self.config.player.sound.footstep_diameter =
                            crate::constants::cs2::SOUND_ESP_FOOTSTEP_DIAMETER_DEFAULT;
                        self.send_config_game();
                    }
                    if response.changed() {
                        self.send_config_game();
                    }
                });

                ui.horizontal(|ui| {
                    let response = ui.add(
                        egui::DragValue::new(&mut self.config.player.sound.gunshot_diameter)
                            .speed(10.0)
                            .range(200.0..=10000.0),
                    );

                    ui.label("Gunshot");

                    if ui.button("↺").on_hover_text("Reset").clicked() {
                        self.config.player.sound.gunshot_diameter =
                            crate::constants::cs2::SOUND_ESP_GUNSHOT_DIAMETER_DEFAULT;
                        self.send_config_game();
                    }
                    if response.changed() {
                        self.send_config_game();
                    }
                });

                ui.horizontal(|ui| {
                    let response = ui.add(
                        egui::DragValue::new(&mut self.config.player.sound.weapon_diameter)
                            .speed(10.0)
                            .range(200.0..=6000.0),
                    );

                    ui.label("Weapon");

                    if ui.button("↺").on_hover_text("Reset").clicked() {
                        self.config.player.sound.weapon_diameter =
                            crate::constants::cs2::SOUND_ESP_WEAPON_DIAMETER_DEFAULT;
                        self.send_config_game();
                    }
                    if response.changed() {
                        self.send_config_game();
                    }
                });
            });
        });
    }
}
