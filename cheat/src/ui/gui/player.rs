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

fn color_values_editor(ui: &mut Ui, colors: &mut ColorValues) -> bool {
    let mut changed = false;
    changed |= color_picker(ui, "Enemy", &mut colors.enemy_color);
    changed |= color_picker(ui, "Ally", &mut colors.ally_color);
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
        "C4 Color Enabled",
        "Overrides whatever color the mode above would use for the bomb carrier. Model/Hitbox ESP dim it while they're not visible, same as everyone else.",
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

            collapsing_open(ui, "Advanced", |ui| {
                ui.label(
                    "Shared/global colors, used by any ESP below that hasn't enabled its own override:",
                );
                if color_values_editor(ui, &mut self.config.player.colors) {
                    self.send_config_game();
                }

                ui.collapsing("Box", |ui| {
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

                ui.collapsing("Skeleton", |ui| {
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

                ui.collapsing("Model", |ui| {
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

                ui.collapsing("Hitbox", |ui| {
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

                ui.collapsing("Model Extraction", |ui| {
                    ui.label("Model ESP needs these extracted from your own CS2 install first:");

                    if checkbox_hover(
                        ui,
                        "Auto-Extract on Startup",
                        "Automatically reads your CS2 install to set this up the first time deadlocked runs, no manual button click needed. Off by default: turning on Model ESP will ask instead.",
                        &mut self.config.hud.auto_extract_models,
                    ) {
                        self.send_config_game();
                    }

                    self.mesh_extract_button(ui);
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
                let needs_extraction = !crate::config::BASE_PATH.join("agent_models.json").exists();
                if self.config.player.draw_model != crate::config::player::ModelEspMode::Off
                    && needs_extraction
                {
                    self.extract_prompt = true;
                }
            }

            if combo_box(
                ui,
                "model_color_mode",
                "Model Color",
                &mut self.config.player.model_color_mode,
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

            if combo_box(
                ui,
                "hitbox_color_mode",
                "Hitbox Color",
                &mut self.config.player.hitbox_color_mode,
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

    pub(crate) fn start_mesh_extraction(&self) {
        use crate::mesh_extract::ExtractStatus;

        let status = self.mesh_extract_status.clone();
        *status.lock() = ExtractStatus::Running;
        std::thread::spawn(move || {
            let result = crate::mesh_extract::extract_all_agent_models(&status);
            *status.lock() = match result {
                Ok(path) => ExtractStatus::Done(path),
                Err(err) => ExtractStatus::Error(err),
            };
        });
    }

    fn mesh_extract_button(&mut self, ui: &mut Ui) {
        use crate::mesh_extract::ExtractStatus;

        let running = matches!(
            *self.mesh_extract_status.lock(),
            ExtractStatus::Running | ExtractStatus::Progress { .. }
        );

        ui.horizontal(|ui| {
            ui.add_enabled_ui(!running, |ui| {
                if ui.button("Extract Player Models").clicked() {
                    self.start_mesh_extraction();
                }
            });

            self.mesh_extract_status_line(ui);
        });
    }

    pub(crate) fn mesh_extract_status_line(&self, ui: &mut Ui) {
        use crate::mesh_extract::ExtractStatus;

        match &*self.mesh_extract_status.lock() {
            ExtractStatus::Idle => {}
            ExtractStatus::Running => {
                ui.label("finding your CS2 install and reading agent list...");
            }
            ExtractStatus::Progress {
                done,
                total,
                started_at,
            } => {
                let eta = if *done > 0 {
                    let per_model = started_at.elapsed().as_secs_f32() / *done as f32;
                    let remaining = ((*total - *done) as f32 * per_model).round() as u64;
                    format!(", ~{}s left", remaining)
                } else {
                    String::new()
                };
                ui.label(format!("extracting agent models... {done}/{total}{eta}"));
            }
            ExtractStatus::Done(path) => {
                ui.label(format!("wrote {}", path.display()));
            }
            ExtractStatus::Error(err) => {
                ui.colored_label(egui::Color32::RED, err);
            }
        }
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
