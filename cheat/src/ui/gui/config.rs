use egui::{Align, Button, Ui};

use crate::{
    config::{
        BASE_PATH, CONFIG_PATH, Config, available_configs, delete_config, parse_config,
        write_config,
    },
    ui::{
        app::App,
        color::Colors,
        grenades::read_grenades,
        gui::helpers::{collapsing_open, scroll},
    },
};

impl App {
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
                std::process::Command::new("xdg-open")
                    .arg(BASE_PATH.as_os_str())
                    .status()
                    .unwrap();
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
                            ui.ctx()
                                .global_style_mut(|style| style.visuals.selection.bg_fill = color);
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
