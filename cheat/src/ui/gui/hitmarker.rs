use egui::{DragValue, RichText, Ui};

use crate::{
    config::hud::HitmarkerSound,
    ui::{
        app::AppState,
        color::Colors,
        gui::helpers::{checkbox, collapsing_open, color_picker, combo_box, drag, scroll},
    },
};

impl AppState {
    pub fn hitmarker_settings(&mut self, ui: &mut Ui) {
        scroll(ui, "hitmarker", |ui| {
            ui.columns(2, |columns| {
                self.hitmarker_left(&mut columns[0]);
                self.hitmarker_right(&mut columns[1]);
            });

            collapsing_open(ui, "Colors", |ui| {
                if color_picker(ui, "Hit Color", &mut self.config.hud.hitmarker.color) {
                    self.send_config_game();
                }
                if color_picker(ui, "Kill Color", &mut self.config.hud.hitmarker.kill_color) {
                    self.send_config_game();
                }
            });
        });
    }

    fn hitmarker_left(&mut self, ui: &mut Ui) {
        collapsing_open(ui, "Hitmarker", |ui| {
            if checkbox(
                ui,
                "Enable Hitmarker",
                &mut self.config.hud.hitmarker.enabled,
            ) {
                self.send_config_game();
            }
            if drag(
                ui,
                "Duration",
                DragValue::new(&mut self.config.hud.hitmarker.duration)
                    .range(0.01..=2.0)
                    .suffix(" s")
                    .speed(0.01),
            ) {
                self.send_config_game();
            }
        });

        collapsing_open(ui, "Appearance", |ui| {
            if drag(
                ui,
                "Line Length",
                DragValue::new(&mut self.config.hud.hitmarker.line_length)
                    .range(0.1..=100.0)
                    .speed(0.2),
            ) {
                self.send_config_game();
            }
            if drag(
                ui,
                "Line Width",
                DragValue::new(&mut self.config.hud.hitmarker.line_width)
                    .range(0.1..=20.0)
                    .speed(0.1),
            ) {
                self.send_config_game();
            }
            if drag(
                ui,
                "Gap",
                DragValue::new(&mut self.config.hud.hitmarker.gap)
                    .range(0.0..=100.0)
                    .speed(0.2),
            ) {
                self.send_config_game();
            }
        });
    }

    fn hitmarker_right(&mut self, ui: &mut Ui) {
        collapsing_open(ui, "Sound", |ui| {
            if checkbox(
                ui,
                "Enable Sound",
                &mut self.config.hud.hitmarker.sound_enabled,
            ) {
                self.send_config_game();
            }

            let sound_changed = combo_box(
                ui,
                "hitmarker_sound",
                "Sound",
                &mut self.config.hud.hitmarker.sound,
            );
            if sound_changed {
                self.reload_custom_sound();
                self.send_config_game();
            }

            if self.config.hud.hitmarker.sound == HitmarkerSound::Custom {
                ui.horizontal(|ui| {
                    ui.label("Custom Path");
                    ui.text_edit_singleline(&mut self.config.hud.hitmarker.custom_sound_path);
                });
                if ui.button("Load").clicked() {
                    self.reload_custom_sound();
                    self.send_config_game();
                }
                if let Some(warning) = &self.custom_sound_warning {
                    ui.label(RichText::new(warning).color(Colors::YELLOW));
                }
            }

            ui.horizontal(|ui| {
                if ui.button("Test Sound").clicked() {
                    self.play_hitmarker_test();
                }
                ui.label("Preview selected sound");
            });

            if drag(
                ui,
                "Volume",
                DragValue::new(&mut self.config.hud.hitmarker.volume)
                    .range(0.0..=1.0)
                    .speed(0.01)
                    .max_decimals(2),
            ) {
                self.send_config_game();
            }
        });
    }
}
