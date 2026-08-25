use egui::{Align, Button, Ui, Color32};

use crate::{
    config::{
        BASE_PATH, CONFIG_PATH, Config, available_configs, delete_config, parse_config,
        write_config,
    },
    ui::{
        app::AppState,
        color::Colors,
        grenades::read_grenades,
        gui::helpers::{collapsing_open, open_url, scroll, color_picker},
    },
};

impl AppState {
    pub fn config_settings(&mut self, ui: &mut Ui) {
        ui.columns(2, |cols| {
            let left = &mut cols[0];
            scroll(left, "config_left", |left| self.config_left(left));

            let right = &mut cols[1];

            collapsing_open(right, "Configs", |right| {
                if right.button("Refresh").clicked() {
                    self.available_configs = available_configs();
                    self.grenades = read_grenades();
                }

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

                scroll(right, "config_right", |right| self.config_right(right));
            });
        });
    }

    fn config_left(&mut self, ui: &mut Ui) {
        collapsing_open(ui, "Config", |ui| {
            if ui.button("Reset").clicked() {
                self.config = Config::default();
                self.send_config();
                utils::info!("loaded default config");
            }

            if ui.button("Config Folder").clicked() {
                let url = format!("file://{}", BASE_PATH.display());
                open_url(&url);
            }
        });

        collapsing_open(ui, "Accent Color", |ui| {
            let mut color = self.config.accent_color;
            let mut updated = false;
            let selected_name = Colors::ACCENT_COLORS
                .iter()
                .find(|(_, c)| *c == color)
                .map(|(name, _)| *name);

            egui::ComboBox::new("accent_color", "Accent Color")
                .selected_text(selected_name.unwrap_or("Custom"))
                .show_ui(ui, |ui| {
                    for (name, preset_color) in Colors::ACCENT_COLORS {
                        if ui.selectable_value(&mut color, preset_color, name).clicked() {
                            updated = true;
                        }
                    }
                    if ui.selectable_label(selected_name.is_none(), "Custom").clicked() {
                        color = Color32::from_gray(128);
                        updated = true;
                    }
                });

            if selected_name.is_none() {
                if color_picker(ui, "Custom Color", &mut color) {
                    updated = true;
                }
            }

            if updated {
                self.config.accent_color = color;
                self.send_config();
                ui.ctx().global_style_mut(|s| s.visuals.selection.bg_fill = color);
            }
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
                    if ui.button("Delete").clicked() {
                        delete = Some(config.clone());
                    }
                });
            });
        }

        if let Some(config_path) = clicked_config {
            self.config = parse_config(&config_path);
            self.current_config = config_path;
            self.send_config();
            ui.ctx().global_style_mut(|style| {
                style.visuals.selection.bg_fill = self.config.accent_color
            });
        }

        if let Some(config) = delete {
            delete_config(&config);
            self.available_configs = available_configs();
            self.current_config = self.available_configs[0].clone();
            self.config = parse_config(&self.current_config);
        }
    }
}
