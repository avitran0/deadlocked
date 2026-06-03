use egui::{DragValue, Ui};

use crate::{
    constants::cs2::GRENADES,
    ui::{
        app::App,
        color::Colors,
        grenades::{Grenade, write_grenades},
        gui::helpers::{checkbox, collapsing_open, drag, keybind, scroll},
    },
};

impl App {
    pub fn grenade_settings(&mut self, ui: &mut Ui) {
        scroll(ui, "hud", |ui| {
            self.automation_settings(ui);

            if self.current_grenade.is_some() {
                self.edit_grenade(ui);
            } else {
                self.record_grenade(ui);
            }

            // grenade list
            ui.collapsing("Grenade List", |ui| {
                self.grenade_list(ui);
            });
        });
    }

    fn automation_settings(&mut self, ui: &mut Ui) {
        collapsing_open(ui, "Automation", |ui| {
            if checkbox(
                ui,
                "Enable Automation",
                &mut self.config.grenade.automation_enabled,
            ) {
                self.send_config();
            }
            if keybind(
                ui,
                "grenade_automation_hotkey",
                "Automation Hotkey",
                &mut self.config.grenade.automation_hotkey,
            ) {
                self.send_config();
            }
            if drag(
                ui,
                "Activation Distance",
                DragValue::new(&mut self.config.grenade.activation_distance)
                    .range(1.0..=128.0)
                    .speed(1.0),
            ) {
                self.send_config();
            }
            if drag(
                ui,
                "Marker Draw Distance",
                DragValue::new(&mut self.config.grenade.marker_draw_distance)
                    .range(24.0..=2000.0)
                    .speed(10.0),
            ) {
                self.send_config();
            }
            if drag(
                ui,
                "Position Tolerance",
                DragValue::new(&mut self.config.grenade.position_tolerance)
                    .range(0.5..=24.0)
                    .speed(0.25),
            ) {
                self.send_config();
            }
            if drag(
                ui,
                "Aim FOV",
                DragValue::new(&mut self.config.grenade.aim_fov)
                    .range(1.0..=180.0)
                    .speed(1.0),
            ) {
                self.send_config();
            }
            if drag(
                ui,
                "Aim Smooth",
                DragValue::new(&mut self.config.grenade.aim_smooth)
                    .range(0.0..=20.0)
                    .speed(0.25),
            ) {
                self.send_config();
            }
            if drag(
                ui,
                "Aim Inertia",
                DragValue::new(&mut self.config.grenade.aim_inertia)
                    .range(0.0..=1.0)
                    .speed(0.005)
                    .max_decimals(2),
            ) {
                self.send_config();
            }
            if drag(
                ui,
                "Aim Curve",
                DragValue::new(&mut self.config.grenade.aim_curve)
                    .range(0.0..=1.0)
                    .speed(0.005)
                    .max_decimals(2),
            ) {
                self.send_config();
            }
            if drag(
                ui,
                "Aim Humanization",
                DragValue::new(&mut self.config.grenade.aim_humanization)
                    .range(0.0..=1.0)
                    .speed(0.005)
                    .max_decimals(2),
            ) {
                self.send_config();
            }
            if drag(
                ui,
                "Aim Tolerance",
                DragValue::new(&mut self.config.grenade.aim_tolerance)
                    .range(0.01..=2.0)
                    .speed(0.01),
            ) {
                self.send_config();
            }
        });
    }

    fn grenade_list(&mut self, ui: &mut Ui) {
        let mut should_write = false;

        for (map, grenades) in self.grenades.iter_mut() {
            let mut delete_grenade_index = None;

            ui.collapsing(map, |ui| {
                for (index, grenade) in grenades.iter().enumerate() {
                    let active = match &self.current_grenade {
                        Some(grenade) => &grenade.0 == map && grenade.1 == index,
                        None => false,
                    };
                    ui.horizontal(|ui| {
                        if ui.selectable_label(active, &grenade.name).clicked() {
                            self.current_grenade = match self.current_grenade {
                                Some((ref g_map, ref g_index))
                                    if g_map == map && *g_index == index =>
                                {
                                    None
                                }
                                _ => Some((map.to_owned(), index)),
                            };
                        }
                        if ui.button("\u{f0a7a}").clicked() {
                            delete_grenade_index = Some(index);
                        }
                    });
                }
                if let Some(index) = delete_grenade_index {
                    grenades.remove(index);
                    should_write = true;
                }
            });
        }

        if should_write {
            write_grenades(&self.grenades);
            self.send_config();
        }
    }

    fn record_grenade(&mut self, ui: &mut Ui) {
        collapsing_open(ui, "Add Grenade", |ui| {
            let (in_game, map, position, view_angles, weapon) = {
                let data = self.data.lock();
                (
                    data.in_game,
                    data.map_name.clone(),
                    data.local_player.position,
                    data.view_angles,
                    data.local_player.weapon.clone(),
                )
            };

            if !in_game {
                ui.label("Not in game.");
                return;
            }

            let grenade = if !GRENADES.contains(&weapon) {
                ui.colored_label(Colors::YELLOW, "Invalid Weapon");
                return;
            } else {
                &weapon
            };

            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut self.new_grenade.name);
                ui.label("Name");
            });

            ui.horizontal(|ui| {
                ui.text_edit_multiline(&mut self.new_grenade.description);
                ui.label("Instructions");
            });

            ui.checkbox(&mut self.new_grenade.modifiers.jump, "Jump");
            ui.checkbox(&mut self.new_grenade.modifiers.duck, "Duck");
            ui.checkbox(&mut self.new_grenade.modifiers.run, "Run");
            self.grenade_automation_editor(ui, true);

            if ui.button("Save").clicked() {
                let grenade_list = match self.grenades.get_mut(&map) {
                    Some(list) => list,
                    None => {
                        self.grenades.insert(map.clone(), Vec::new());
                        self.grenades.get_mut(&map).unwrap()
                    }
                };

                let mut new_grenade = Grenade::new();
                std::mem::swap(&mut new_grenade, &mut self.new_grenade);

                new_grenade.weapon = grenade.clone();
                new_grenade.position = position;
                new_grenade.view_angles = view_angles;

                grenade_list.push(new_grenade);
                write_grenades(&self.grenades);
                self.send_config();
            }
        });
    }

    fn edit_grenade(&mut self, ui: &mut Ui) {
        collapsing_open(ui, "Edit Grenade", |ui| {
            let (map, index) = match &self.current_grenade {
                Some(grenade) => grenade,
                None => return,
            };

            let Some(grenades) = self.grenades.get_mut(map) else {
                return;
            };
            let Some(grenade) = grenades.get_mut(*index) else {
                return;
            };

            let mut changed = ui
                .horizontal(|ui| {
                    let response = ui.text_edit_singleline(&mut grenade.name);
                    ui.label("Name");
                    response
                })
                .inner
                .changed();

            changed |= ui
                .horizontal(|ui| {
                    let response = ui.text_edit_multiline(&mut grenade.description);
                    ui.label("Description");
                    response
                })
                .inner
                .changed();

            changed |= ui.checkbox(&mut grenade.modifiers.jump, "Jump").changed();
            changed |= ui.checkbox(&mut grenade.modifiers.duck, "Duck").changed();
            changed |= ui.checkbox(&mut grenade.modifiers.run, "Run").changed();

            changed |= ui
                .collapsing("Automation Data", |ui| {
                    let mut changed = false;
                    changed |= ui
                        .checkbox(&mut grenade.automation.always_run, "Always Run")
                        .changed();
                    changed |= drag(
                        ui,
                        "Movement Flags",
                        DragValue::new(&mut grenade.automation.movement_flags).range(0..=2048),
                    );
                    changed |= drag(
                        ui,
                        "Run Ticks",
                        DragValue::new(&mut grenade.automation.run_ticks).range(0..=300),
                    );
                    changed |= drag(
                        ui,
                        "Jump Delay Ticks",
                        DragValue::new(&mut grenade.automation.jump_delay_ticks).range(0..=300),
                    );
                    changed |= drag(
                        ui,
                        "Throw Strength",
                        DragValue::new(&mut grenade.automation.throw_strength)
                            .range(0.0..=1.0)
                            .speed(0.01),
                    );
                    changed
                })
                .body_returned
                .unwrap_or(false);

            if changed {
                write_grenades(&self.grenades);
                self.send_config();
            }
        });
    }

    fn grenade_automation_editor(&mut self, ui: &mut Ui, new_grenade: bool) {
        ui.collapsing("Automation Data", |ui| {
            let automation = if new_grenade {
                &mut self.new_grenade.automation
            } else {
                return;
            };

            ui.checkbox(&mut automation.always_run, "Always Run");
            drag(
                ui,
                "Movement Flags",
                DragValue::new(&mut automation.movement_flags).range(0..=2048),
            );
            drag(
                ui,
                "Run Ticks",
                DragValue::new(&mut automation.run_ticks).range(0..=300),
            );
            drag(
                ui,
                "Jump Delay Ticks",
                DragValue::new(&mut automation.jump_delay_ticks).range(0..=300),
            );
            drag(
                ui,
                "Throw Strength",
                DragValue::new(&mut automation.throw_strength)
                    .range(0.0..=1.0)
                    .speed(0.01),
            );
        });
    }
}
