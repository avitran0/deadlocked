use egui::{DragValue, Ui};

use crate::{
    config::r#unsafe::UnsafeConfig,
    ui::{
        app::AppState,
        gui::helpers::{collapsing_open, color_picker, reset_button},
    },
};

impl AppState {
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
                self.send_config_game();
            }

            if ui
                .checkbox(
                    &mut self.config.misc.change_smoke_color,
                    "Change Smoke Color",
                )
                .changed()
            {
                self.send_config_game();
            }

            if color_picker(ui, "Smoke Color", &mut self.config.misc.smoke_color) {
                self.send_config_game();
            }

            if reset_button(ui) {
                let defaults = UnsafeConfig::default();
                self.config.misc.no_smoke = defaults.no_smoke;
                self.config.misc.change_smoke_color = defaults.change_smoke_color;
                self.config.misc.smoke_color = defaults.smoke_color;
                self.send_config_game();
            }
        });
    }

    fn unsafe_left(&mut self, ui: &mut Ui) {
        collapsing_open(ui, "No Flash", |ui| {
            if ui
                .checkbox(&mut self.config.misc.no_flash, "No Flash")
                .changed()
            {
                self.send_config_game();
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
                    self.send_config_game();
                }
                ui.label("Max Flash Alpha");
            });

            if reset_button(ui) {
                let defaults = UnsafeConfig::default();
                self.config.misc.no_flash = defaults.no_flash;
                self.config.misc.max_flash_alpha = defaults.max_flash_alpha;
                self.send_config_game();
            }
        });
    }

    fn unsafe_right(&mut self, ui: &mut Ui) {
        collapsing_open(ui, "FOV Changer", |ui| {
            if ui
                .checkbox(&mut self.config.misc.fov_changer, "FOV Changer")
                .changed()
            {
                self.send_config_game();
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
                    self.send_config_game();
                }
                ui.label("Desired FOV");

                if ui.button("Reset").clicked() {
                    self.config.misc.desired_fov = crate::constants::cs2::DEFAULT_FOV;
                    self.send_config_game();
                }
            });

            if reset_button(ui) {
                let defaults = UnsafeConfig::default();
                self.config.misc.fov_changer = defaults.fov_changer;
                self.config.misc.desired_fov = defaults.desired_fov;
                self.send_config_game();
            }
        });
    }
}
