use egui::{Align, CollapsingHeader, Color32, Context, DragValue, Ui};

use crate::{
    config::{BASE_PATH, VERSION, WeaponConfig, write_config},
    message::{Envelope, GameStatus, Message, Target},
    os::{
        crash::report_error,
        mouse::{DeviceStatus, discover_mice},
    },
    ui::{app::App, color::Colors, gui::aimbot::AimbotTab},
};

pub mod aimbot;
mod config;
mod grenade;
mod hud;
mod player;
mod radar;
mod r#unsafe;

#[derive(PartialEq)]
pub enum Tab {
    Aimbot,
    Player,
    Hud,
    Radar,
    Grenades,
    Unsafe,
    Config,
}

impl App {
    pub fn send_config(&self) {
        self.send_message(Message::Config(Box::new(self.config.clone())), Target::Game);
        self.save();
    }

    pub fn send_message(&self, message: Message, target: Target) {
        if self.tx.send(Envelope { target, message }).is_err() {
            std::process::exit(1);
        }
    }

    fn save(&self) {
        write_config(&self.config, &self.current_config);
    }

    fn gui(&mut self, ctx: &Context) {
        ctx.set_pixels_per_point(self.display_scale);
        egui::SidePanel::left("sidebar")
            .resizable(false)
            .show(ctx, |ui| {
                ui.selectable_value(&mut self.current_tab, Tab::Aimbot, "\u{f04fe} Aimbot");
                ui.selectable_value(&mut self.current_tab, Tab::Player, "\u{f0013} Player");
                ui.selectable_value(&mut self.current_tab, Tab::Hud, "\u{f0379} Hud");
                ui.selectable_value(&mut self.current_tab, Tab::Radar, "\u{f0437} Radar");
                ui.selectable_value(&mut self.current_tab, Tab::Grenades, "\u{f0691} Grenades");
                ui.selectable_value(&mut self.current_tab, Tab::Unsafe, "\u{f0ce6} Unsafe");
                ui.selectable_value(&mut self.current_tab, Tab::Config, "\u{f168b} Config");

                ui.with_layout(egui::Layout::bottom_up(Align::Min), |ui| {
                    if ui.button("Report Issue").clicked() {
                        std::process::Command::new("xdg-open")
                            .arg("https://github.com/avitran0/deadlocked/issues")
                            .status()
                            .unwrap();
                    }
                    if ui.button("Config Folder").clicked() {
                        std::process::Command::new("xdg-open")
                            .arg(BASE_PATH.as_os_str())
                            .status()
                            .unwrap();
                    }
                    ui.label(VERSION);
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.add_game_status(ui);
            ui.separator();

            match self.current_tab {
                Tab::Aimbot => self.aimbot_settings(ui),
                Tab::Player => self.player_settings(ui),
                Tab::Hud => self.hud_settings(ui),
                Tab::Radar => self.radar_settings(ui),
                Tab::Grenades => self.grenade_settings(ui),
                Tab::Unsafe => self.unsafe_settings(ui),
                Tab::Config => self.config_settings(ui, ctx),
            }
        });
    }

    fn weapon_config(&mut self) -> &mut WeaponConfig {
        if self.aimbot_tab == AimbotTab::Weapon {
            self.config
                .aim
                .weapons
                .get_mut(&self.aimbot_weapon)
                .unwrap()
        } else {
            &mut self.config.aim.global
        }
    }

    fn add_game_status(&mut self, ui: &mut Ui) {
        // Top row with status info on the left and a small "Deadlocked" logo/text on the right.
        ui.horizontal_top(|ui| {
            // Left side: original status + mouse combobox
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format!("{}", self.game_status))
                        .line_height(Some(8.0))
                        .color(match self.game_status {
                            GameStatus::Working => Colors::GREEN,
                            GameStatus::GameNotStarted => Colors::YELLOW,
                        }),
                );

                let mouse_text = match &self.mouse_status {
                    DeviceStatus::Working(name) => name,
                    DeviceStatus::PermissionsRequired => {
                        "mouse input only works when user is in input group"
                    }
                    DeviceStatus::Disconnected => "mouse was disconnected",
                    DeviceStatus::NotFound => "no mouse was found",
                };

                let color = match &self.mouse_status {
                    DeviceStatus::Working(_) => Colors::SUBTEXT,
                    _ => Colors::YELLOW,
                };
                ui.label(
                    egui::RichText::new(mouse_text)
                        .line_height(Some(8.0))
                        .color(color),
                );

                egui::ComboBox::new("mouse_device", "")
                    .selected_text(
                        self.selected_mouse
                            .as_deref()
                            .unwrap_or("No device selected"),
                    )
                    .show_ui(ui, |ui| {
                        for device in discover_mice() {
                            let label = format!("{} ({})", device.name, device.event_name);
                            if ui
                                .selectable_label(
                                    self.selected_mouse.as_deref() == Some(&device.event_name),
                                    &label,
                                )
                                .clicked()
                            {
                                self.selected_mouse = Some(device.event_name.clone());

                                self.send_message(
                                    Message::SelectMouse(device.event_name.clone()),
                                    Target::Game,
                                );
                            }
                        }
                    });
            });

            // Right side: small logo/text. Uses the configured accent color so it matches the theme.
            ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                let logo = egui::RichText::new("Deadlocked")
                    .size(16.0)
                    .strong()
                    .color(self.config.accent_color);
                ui.label(logo);
            });
        });
    }

    fn color_picker(&self, ui: &mut Ui, color: &Color32, label: &str) -> Option<Color32> {
        let [mut r, mut g, mut b, _] = color.to_array();
        let res = ui
            .horizontal(|ui| {
                let (response, painter) =
                    ui.allocate_painter(ui.spacing().interact_size, egui::Sense::hover());
                painter.rect_filled(
                    response.rect,
                    ui.style().visuals.widgets.inactive.corner_radius,
                    *color,
                );
                let mut res = ui.add(DragValue::new(&mut r).prefix("r: "));
                res = res.union(ui.add(DragValue::new(&mut g).prefix("g: ")));
                res = res.union(ui.add(DragValue::new(&mut b).prefix("b: ")));
                ui.label(label);
                res
            })
            .inner;

        if res.changed() {
            Some(Color32::from_rgb(r, g, b))
        } else {
            None
        }
    }

    pub fn render(&mut self) {
        let self_ptr = self as *mut Self;

        let gui = self.gui.as_mut().unwrap();

        if let Err(err) = gui.make_current() {
            log::error!("could not make gui window current: {err}");
            report_error(err);
            return;
        }
        gui.run(|ctx| (unsafe { &mut *self_ptr }).gui(ctx));
        gui.clear();
        gui.paint();

        if let Err(err) = gui.swap_buffers() {
            log::error!("could not swap gui window buffers: {err}");
            report_error(err);
            return;
        }

        let overlay = self.overlay.as_mut().unwrap();

        overlay.window().set_cursor_hittest(false).unwrap();
        if let Err(err) = overlay.make_current() {
            log::error!("could not make overlay window current: {err}");
            report_error(err);
            return;
        }

        overlay.run(move |egui_ctx| {
            (unsafe { &mut *self_ptr }).overlay(egui_ctx);
        });
        overlay.clear();
        overlay.paint();

        if let Err(err) = overlay.swap_buffers() {
            log::error!("could not swap overlay window buffers: {err}");
            report_error(err);
        }
    }
}

fn collapsing_open(ui: &mut Ui, title: &str, add_body: impl FnOnce(&mut Ui)) {
    CollapsingHeader::new(title)
        .default_open(true)
        .show(ui, add_body);
}
