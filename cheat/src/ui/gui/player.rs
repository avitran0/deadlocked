use egui::{DragValue, Ui};

use crate::{
    config::player::{BehindPlayersConfig, PlayerConfig, SoundConfig},
    ui::{
        app::AppState,
        gui::helpers::{
            checkbox, checkbox_hover, collapsing_open, color_picker, combo_box, drag, keybind,
            reset_button, scroll, text_settings_button,
        },
    },
};

impl AppState {
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
                    self.send_config_game();
                }

                if color_picker(
                    ui,
                    "Box (invisible)",
                    &mut self.config.player.box_invisible_color,
                ) {
                    self.send_config_game();
                }

                if color_picker(ui, "Skeleton", &mut self.config.player.skeleton_color) {
                    self.send_config_game();
                }

                if reset_button(ui) {
                    let defaults = PlayerConfig::default();
                    self.config.player.box_visible_color = defaults.box_visible_color;
                    self.config.player.box_invisible_color = defaults.box_invisible_color;
                    self.config.player.skeleton_color = defaults.skeleton_color;
                    self.send_config_game();
                }
            });
        });
    }

    fn player_left(&mut self, ui: &mut Ui) {
        collapsing_open(ui, "Players", |ui| {
            if checkbox(ui, "Player", &mut self.config.player.enabled) {
                self.send_config_game();
            }

            if checkbox(ui, "Chicken", &mut self.config.player.chicken) {
                self.send_config_game();
            }

            if keybind(
                ui,
                "esp_hotkey",
                "ESP Hotkey",
                &mut self.config.player.esp_hotkey,
            ) {
                self.send_config_game();
            }

            if checkbox(
                ui,
                "Show Friendlies",
                &mut self.config.player.show_friendlies,
            ) {
                self.send_config_game();
            }

            if combo_box(ui, "draw_box", "Box", &mut self.config.player.draw_box) {
                self.send_config_game();
            }

            if combo_box(ui, "box_mode", "Box Mode", &mut self.config.player.box_mode) {
                self.send_config_game();
            }

            if combo_box(
                ui,
                "draw_skeleton",
                "Skeleton",
                &mut self.config.player.draw_skeleton,
            ) {
                self.send_config_game();
            }

            if checkbox(ui, "Head Circle", &mut self.config.player.head_circle) {
                self.send_config_game();
            }

            if checkbox_hover(
                ui,
                "Visible Only",
                "Only show visible players",
                &mut self.config.player.visible_only,
            ) {
                self.send_config_game();
            }

            if reset_button(ui) {
                let defaults = PlayerConfig::default();
                let player = &mut self.config.player;
                player.enabled = defaults.enabled;
                player.chicken = defaults.chicken;
                player.esp_hotkey = defaults.esp_hotkey;
                player.show_friendlies = defaults.show_friendlies;
                player.draw_box = defaults.draw_box;
                player.box_mode = defaults.box_mode;
                player.draw_skeleton = defaults.draw_skeleton;
                player.head_circle = defaults.head_circle;
                player.visible_only = defaults.visible_only;
                self.send_config_game();
            }
        });

        ui.collapsing("Behind Players", |ui| {
            if checkbox_hover(
                ui,
                "Enabled",
                "Show markers for enemies outside your view (behind, left, right)",
                &mut self.config.player.behind_players.enabled,
            ) {
                self.send_config_game();
            }

            if drag(
                ui,
                "Size",
                DragValue::new(&mut self.config.player.behind_players.size)
                    .range(10.0..=48.0)
                    .max_decimals(0)
                    .speed(0.5),
            ) {
                self.send_config_game();
            }

            if color_picker(
                ui,
                "Hidden Color",
                &mut self.config.player.behind_players.color,
            ) {
                self.send_config_game();
            }

            if color_picker(
                ui,
                "Visible Color",
                &mut self.config.player.behind_players.visible_color,
            ) {
                self.send_config_game();
            }

            if reset_button(ui) {
                self.config.player.behind_players = BehindPlayersConfig::default();
                self.send_config_game();
            }
        });
    }

    fn player_right(&mut self, ui: &mut Ui) {
        collapsing_open(ui, "Info", |ui| {
            if ui
                .checkbox(&mut self.config.player.health_bar, "Health Bar")
                .changed()
            {
                self.send_config_game();
            }

            if ui
                .checkbox(&mut self.config.player.armor_bar, "Armor Bar")
                .changed()
            {
                self.send_config_game();
            }

            ui.horizontal(|ui| {
                if ui
                    .checkbox(&mut self.config.player.player_name, "Player Name")
                    .changed()
                {
                    self.send_config_game();
                }
                text_settings_button(ui, &mut self.text_popup, "player_name");
            });

            ui.horizontal(|ui| {
                if ui
                    .checkbox(&mut self.config.player.weapon_icon, "Weapon Icon")
                    .changed()
                {
                    self.send_config_game();
                }
                text_settings_button(ui, &mut self.text_popup, "weapon_icon");
            });

            ui.horizontal(|ui| {
                ui.label("Ammo");
                text_settings_button(ui, &mut self.text_popup, "ammo_text");
            });

            ui.horizontal(|ui| {
                if ui
                    .checkbox(&mut self.config.player.tags, "Show Tags")
                    .changed()
                {
                    self.send_config_game();
                }
                text_settings_button(ui, &mut self.text_popup, "player_tags");
            });

            if reset_button(ui) {
                let defaults = PlayerConfig::default();
                let player = &mut self.config.player;
                player.health_bar = defaults.health_bar;
                player.armor_bar = defaults.armor_bar;
                player.player_name = defaults.player_name;
                player.weapon_icon = defaults.weapon_icon;
                player.tags = defaults.tags;
                self.send_config_game();
            }
        });

        ui.collapsing("Sound ESP", |ui| {
            if checkbox_hover(
                ui,
                "Enabled",
                "Show a circle under players when they make sound",
                &mut self.config.player.sound.enabled,
            ) {
                self.send_config_game();
            }

            if drag(
                ui,
                "Fadeout Time (s)",
                DragValue::new(&mut self.config.player.sound.fadeout_duration)
                    .range(0.0..=10.0)
                    .speed(0.01),
            ) {
                self.send_config_game();
            }

            if checkbox(
                ui,
                "Show Visible",
                &mut self.config.player.sound.show_visible,
            ) {
                self.send_config_game();
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
                        self.send_config_game();
                    }
                    if response.changed() {
                        self.send_config_game();
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
                        self.send_config_game();
                    }
                    if response.changed() {
                        self.send_config_game();
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
                        self.send_config_game();
                    }
                    if response.changed() {
                        self.send_config_game();
                    }
                });
            });

            if reset_button(ui) {
                self.config.player.sound = SoundConfig::default();
                self.send_config_game();
            }
        });
    }
}
