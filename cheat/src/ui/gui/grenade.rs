use egui::Ui;

use crate::{
    constants::cs2::GRENADES,
    ui::{
        app::App,
        color::Colors,
        grenades::{Grenade, write_grenades},
        gui::helpers::{collapsing_open, scroll},
    },
};

impl App {
    pub fn grenade_settings(&mut self, ui: &mut Ui) {
        scroll(ui, "hud", |ui| {
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
        }
    }

    fn record_grenade(&mut self, ui: &mut Ui) {
        collapsing_open(ui, "Add Grenade", |ui| {
            let data = self.data.lock();

            if !data.in_game {
                ui.label("Not in game.");
                return;
            }

            let grenade = if !GRENADES.contains(&data.local_player.weapon) {
                ui.colored_label(Colors::YELLOW, "Invalid Weapon");
                return;
            } else {
                &data.local_player.weapon
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

            if ui.button("Save").clicked() {
                let map = &data.map_name;
                let grenade_list = match self.grenades.get_mut(map) {
                    Some(list) => list,
                    None => {
                        self.grenades.insert(map.to_owned(), Vec::new());
                        self.grenades.get_mut(map).unwrap()
                    }
                };

                let mut new_grenade = Grenade::new();
                std::mem::swap(&mut new_grenade, &mut self.new_grenade);

                new_grenade.weapon = grenade.clone();
                new_grenade.position = data.local_player.position;
                new_grenade.view_angles = data.view_angles;

                grenade_list.push(new_grenade);
                write_grenades(&self.grenades);
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

            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut grenade.name);
                ui.label("Name");
            });

            ui.horizontal(|ui| {
                ui.text_edit_multiline(&mut grenade.description);
                ui.label("Description");
            });

            ui.checkbox(&mut grenade.modifiers.jump, "Jump");
            ui.checkbox(&mut grenade.modifiers.duck, "Duck");
            ui.checkbox(&mut grenade.modifiers.run, "Run");
        });
    }
}
