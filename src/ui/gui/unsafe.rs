use egui::{DragValue, Ui};

use crate::ui::{
    app::App,
    gui::helpers::{collapsing_open, color_picker},
};

impl App {
    pub fn unsafe_settings(&mut self, ui: &mut Ui) {
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

            if color_picker(ui, "Smoke Color", &mut self.config.misc.smoke_color) {
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
        collapsing_open(ui, "Aim", |ui| {
            if ui
                .checkbox(&mut self.config.misc.silent_aim, "Silent Aim")
                .on_hover_text(
                    "Applies angle writes before shot logic instead of only mouse movement",
                )
                .changed()
            {
                self.send_config();
            }
        });

        collapsing_open(ui, "Privacy", |ui| {
            if ui
                .checkbox(
                    &mut self.config.misc.telemetry_enabled,
                    "Enable Crash Telemetry",
                )
                .on_hover_text(
                    "Sends anonymous crash stack traces and basic system info for debugging",
                )
                .changed()
            {
                self.send_config();
            }
        });

        collapsing_open(ui, "Movement", |ui| {
            if ui
                .checkbox(
                    &mut self.config.misc.auto_counter_strafe,
                    "Auto Counter-Strafe On Shot",
                )
                .changed()
            {
                self.send_config();
            }

            ui.horizontal(|ui| {
                if ui
                    .add(
                        DragValue::new(&mut self.config.misc.counter_strafe_tap_ms)
                            .range(1..=80)
                            .speed(1),
                    )
                    .on_hover_text("How long each opposite movement key tap is held")
                    .changed()
                {
                    self.send_config();
                }
                ui.label("Tap Duration (ms)");
            });

            ui.horizontal(|ui| {
                if ui
                    .add(
                        DragValue::new(&mut self.config.misc.counter_strafe_speed_threshold)
                            .range(0.0..=250.0)
                            .speed(0.5)
                            .max_decimals(1),
                    )
                    .on_hover_text(
                        "Minimum local movement speed before velocity-based counter-strafe taps",
                    )
                    .changed()
                {
                    self.send_config();
                }
                ui.label("Speed Threshold");
            });
        });

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
}
