use egui::Ui;
use strum::IntoEnumIterator as _;

use crate::{
    cs2::key_codes::KeyCode,
    ui::{app::App, gui::collapsing_open},
};

impl App {
    pub fn player_settings(&mut self, ui: &mut Ui) {
        egui::ScrollArea::vertical()
            .auto_shrink([false, true])
            .id_salt("player")
            .show(ui, |ui| {
                ui.columns(2, |cols| {
                    let left = &mut cols[0];
                    self.player_left(left);
                    let right = &mut cols[1];
                    self.player_right(right);
                });

                collapsing_open(ui, "Colors", |ui| {
                    if let Some(color) = self.color_picker(
                        ui,
                        &self.config.player.box_visible_color,
                        "Box (visible)",
                    ) {
                        self.config.player.box_visible_color = color;
                        self.send_config();
                    }

                    if let Some(color) = self.color_picker(
                        ui,
                        &self.config.player.box_invisible_color,
                        "Box (invisible)",
                    ) {
                        self.config.player.box_invisible_color = color;
                        self.send_config();
                    }

                    if let Some(color) =
                        self.color_picker(ui, &self.config.player.skeleton_color, "Skeleton")
                    {
                        self.config.player.skeleton_color = color;
                        self.send_config();
                    }
                    if let Some(color) =
                        self.color_picker(ui, &self.config.player.sound.color, "Sound ESP")
                    {
                        self.config.player.sound.color = color;
                        self.send_config();
                    }
                });
            });
    }

    fn player_left(&mut self, ui: &mut Ui) {
        collapsing_open(ui, "Players", |ui| {
            if ui
                .checkbox(&mut self.config.player.enabled, "Enable")
                .changed()
            {
                self.send_config();
            }

            egui::ComboBox::new("esp_hotkey", "ESP Hotkey")
                .selected_text(format!("{:?}", self.config.player.esp_hotkey))
                .show_ui(ui, |ui| {
                    for key_code in KeyCode::iter() {
                        let text = format!("{:?}", &key_code);
                        if ui
                            .selectable_value(&mut self.config.player.esp_hotkey, key_code, text)
                            .clicked()
                        {
                            self.send_config();
                        }
                    }
                });

            if ui
                .checkbox(&mut self.config.player.show_friendlies, "Show Friendlies")
                .on_hover_text("Only active in custom game modes (workshop/custom maps)")
                .changed()
            {
                self.send_config();
            }

            ui.separator();

            if ui
                .checkbox(&mut self.config.player.only_render_on_sound, "Sound Only ESP")
                .on_hover_text("Only draw players when they make noise (steps, shooting)")
                .changed()
            {
                self.send_config();
            }

            if self.config.player.only_render_on_sound {
                ui.horizontal(|ui| {
                    if ui
                        .add(
                            egui::DragValue::new(&mut self.config.player.sound_fadeout_time)
                                .range(0.1..=10.0)
                                .speed(0.1)
                                .suffix("s"),
                        )
                        .on_hover_text("How long visuals linger after sound stops")
                        .changed()
                    {
                        self.send_config();
                    }
                    ui.label("Fade Time");
                });
            }

            ui.separator();

            egui::ComboBox::new("draw_box", "Box")
                .selected_text(format!("{:?}", self.config.player.draw_box))
                .show_ui(ui, |ui| {
                    use crate::config::DrawMode;

                    for mode in DrawMode::iter() {
                        let text = format!("{:?}", &mode);
                        if ui
                            .selectable_value(&mut self.config.player.draw_box, mode, text)
                            .clicked()
                        {
                            self.send_config();
                        }
                    }
                });

            egui::ComboBox::new("box_mode", "Box Mode")
                .selected_text(format!("{:?}", self.config.player.box_mode))
                .show_ui(ui, |ui| {
                    use crate::config::BoxMode;

                    for mode in BoxMode::iter() {
                        let text = format!("{:?}", &mode);
                        if ui
                            .selectable_value(&mut self.config.player.box_mode, mode, text)
                            .clicked()
                        {
                            self.send_config();
                        }
                    }
                });

            egui::ComboBox::new("draw_skeleton", "Skeleton")
                .selected_text(format!("{:?}", self.config.player.draw_skeleton))
                .show_ui(ui, |ui| {
                    use crate::config::DrawMode;

                    for mode in DrawMode::iter() {
                        let text = format!("{:?}", &mode);
                        if ui
                            .selectable_value(&mut self.config.player.draw_skeleton, mode, text)
                            .clicked()
                        {
                            self.send_config();
                        }
                    }
                });

            if ui
                .checkbox(&mut self.config.player.head_circle, "Head Circle")
                .changed()
            {
                self.send_config();
            }
        });
        ui.collapsing("Sound ESP", |ui| {
            if ui
                .checkbox(&mut self.config.player.sound.enabled, "Enabled")
                .on_hover_text("Show a circle under players when they make sound")
                .changed()
            {
                self.send_config();
            }

            ui.horizontal(|ui| {
                let response = ui.add(
                    egui::DragValue::new(&mut self.config.player.sound.circle_scale)
                        .speed(0.1)
                        .range(0.1..=3.0),
                );

                ui.label("Scale");

                if ui.button("↺").on_hover_text("Reset").clicked() {
                    self.config.player.sound.circle_scale = 1.0;
                    self.send_config();
                }

                if response.changed() {
                    self.send_config();
                }
            });

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
    }
}
