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
