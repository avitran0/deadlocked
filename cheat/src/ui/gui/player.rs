use egui::{DragValue, Ui};

use crate::ui::{
    app::App,
    gui::helpers::{
        checkbox, checkbox_hover, collapsing_open, color_picker, combo_box, drag, keybind, scroll,
    },
};

impl App {
    pub fn player_settings(&mut self, ui: &mut Ui) {
        scroll(ui, "player", |ui| {
            ui.columns(2, |cols| {
                let left = &mut cols[0];
                self.player_left(left);
                let right = &mut cols[1];
                self.player_right(right);
            });

            collapsing_open(ui, "Colors", |ui| {
                if color_picker(
                    ui,
                    "Box (visible)",
                    &mut self.config.player.box_visible_color,
                ) {
                    self.send_config();
                }

                if color_picker(
                    ui,
                    "Box (invisible)",
                    &mut self.config.player.box_invisible_color,
                ) {
                    self.send_config();
                }

                if color_picker(ui, "Skeleton", &mut self.config.player.skeleton_color) {
                    self.send_config();
                }
            });
        });
    }

    fn player_left(&mut self, ui: &mut Ui) {
        collapsing_open(ui, "Players", |ui| {
            if checkbox(ui, "Enable", &mut self.config.player.enabled) {
                self.send_config();
            }

            if keybind(
                ui,
                "esp_hotkey",
                "ESP Hotkey",
                &mut self.config.player.esp_hotkey,
            ) {
                self.send_config();
            }

            if checkbox(
                ui,
                "Show Friendlies",
                &mut self.config.player.show_friendlies,
            ) {
                self.send_config();
            }

            if combo_box(ui, "draw_box", "Box", &mut self.config.player.draw_box) {
                self.send_config();
            }

            if combo_box(ui, "box_mode", "Box Mode", &mut self.config.player.box_mode) {
                self.send_config();
            }

            if combo_box(
                ui,
                "draw_skeleton",
                "Skeleton",
                &mut self.config.player.draw_skeleton,
            ) {
                self.send_config();
            }

            if checkbox(ui, "Head Circle", &mut self.config.player.head_circle) {
                self.send_config();
            }

            if checkbox_hover(
                ui,
                "Visible Only",
                "Only show visible players",
                &mut self.config.player.visible_only,
            ) {
                self.send_config();
            }
        });
    }

    fn player_right(&mut self, ui: &mut Ui) {
        collapsing_open(ui, "Info", |ui| {
            if ui
                .checkbox(&mut self.config.player.health_bar, "Health Bar")
                .changed()
            {
                self.send_config();
            }

            if ui
                .checkbox(&mut self.config.player.armor_bar, "Armor Bar")
                .changed()
            {
                self.send_config();
            }

            if ui
                .checkbox(&mut self.config.player.player_name, "Player Name")
                .changed()
            {
                self.send_config();
            }

            if ui
                .checkbox(&mut self.config.player.weapon_icon, "Weapon Icon")
                .changed()
            {
                self.send_config();
            }

            if ui
                .checkbox(&mut self.config.player.tags, "Show Tags")
                .changed()
            {
                self.send_config();
            }
        });

        ui.collapsing("Sound ESP", |ui| {
            if checkbox_hover(
                ui,
                "Enabled",
                "Show a circle under players when they make sound",
                &mut self.config.player.sound.enabled,
            ) {
                self.send_config();
            }

            if drag(
                ui,
                "Fadeout Time (s)",
                DragValue::new(&mut self.config.player.sound.fadeout_duration)
                    .range(0.0..=10.0)
                    .speed(0.01),
            ) {
                self.send_config();
            }

            if checkbox(
                ui,
                "Show Visible",
                &mut self.config.player.sound.show_visible,
            ) {
                self.send_config();
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
                        self.send_config();
                    }
                    if response.changed() {
                        self.send_config();
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
                        self.send_config();
                    }
                    if response.changed() {
                        self.send_config();
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
                        self.send_config();
                    }
                    if response.changed() {
                        self.send_config();
                    }
                });
            });
        });
    }
}
