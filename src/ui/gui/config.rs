use egui::{Align, Button, Context, Slider, Ui};

use crate::{
    config::{CONFIG_PATH, Config, available_configs, delete_config, parse_config, write_config},
    ui::{app::App, color::Colors, gui::collapsing_open},
};

impl App {
    pub fn config_settings(&mut self, ui: &mut Ui, ctx: &Context) {
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

        // Logo animation settings
        collapsing_open(ui, "Logo", |ui| {
            if ui
                .checkbox(&mut self.logo_animated, "Animate Logo")
                .on_hover_text("Animate the top-right logo (blend between two colors)")
                .changed()
            {
                // restart animation
                self.logo_animation_start = std::time::Instant::now();
            }

            // Logo text input
            ui.horizontal(|ui| {
                ui.label("Logo Text");
                if ui.text_edit_singleline(&mut self.logo_text).changed() {
                    // restart animation timing and keep it visible immediately
                    self.logo_animation_start = std::time::Instant::now();
                    // clamp split index to text length
                    let len = self.logo_text.chars().count() as i32;
                    if self.logo_split_index > len {
                        self.logo_split_index = len;
                    }
                }
            });

            // Manual split toggle + slider
            ui.horizontal(|ui| {
                if ui
                    .checkbox(&mut self.logo_manual_split, "Manual Split")
                    .on_hover_text("Enable to pick how many characters go to the left color")
                    .changed()
                {
                    // when enabling manual, ensure the split index is within range
                    let len = self.logo_text.chars().count() as i32;
                    if self.logo_split_index > len {
                        self.logo_split_index = len;
                    }
                }

                if self.logo_manual_split {
                    let len = self.logo_text.chars().count() as i32;
                    let max = if len >= 0 { len } else { 0 };
                    // slider for split index: number of characters assigned to the LEFT color
                    ui.label("Split (left chars)");
                    if ui
                        .add(Slider::new(&mut self.logo_split_index, 0..=max).text(""))
                        .changed()
                    {
                        self.logo_animation_start = std::time::Instant::now();
                    }
                } else {
                    // show computed default split for information
                    let len = self.logo_text.chars().count();
                    let default_left = (len + 1) / 2;
                    ui.label(format!("Auto split: left {} / right {}", default_left, len - default_left));
                }
            });

            // Color A (applies to the left half)
            ui.horizontal(|ui| {
                ui.label("Color A (left)");
                let mut temp_a = self.logo_color_a;
                if ui.color_edit_button_srgba(&mut temp_a).changed() {
                    self.logo_color_a = temp_a;
                }
            });

            // Color B (applies to the right half)
            ui.horizontal(|ui| {
                ui.label("Color B (right)");
                let mut temp_b = self.logo_color_b;
                if ui.color_edit_button_srgba(&mut temp_b).changed() {
                    self.logo_color_b = temp_b;
                }
            });

            ui.horizontal(|ui| {
                ui.label("Speed (Hz)");
                // A slider is used instead of raw numeric entry
                if ui
                    .add(Slider::new(&mut self.logo_animation_speed, 0.1..=5.0).text(""))
                    .changed()
                {
                    // restart animation cleanly
                    self.logo_animation_start = std::time::Instant::now();
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
}
