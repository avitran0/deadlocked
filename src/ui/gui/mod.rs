use std::time::Duration;

use egui::{Align, Ui};

use crate::{
    config::{aim::WeaponConfig, write_config},
    message::{GameMessage, GameStatus},
    ui::{app::App, color::Colors, gui::aimbot::AimbotTab},
};

pub mod aimbot;
mod application;
mod config;
mod grenade;
mod helpers;
mod hud;
mod player;
mod r#unsafe;

#[derive(PartialEq)]
pub enum Tab {
    Aimbot,
    Player,
    Hud,
    Grenades,
    Unsafe,
    Config,
    Application,
}

impl App {
    pub fn send_config(&self) {
        self.send_message(GameMessage(Box::new(self.config.clone())));
        self.save();
    }

    pub fn send_message(&self, message: GameMessage) {
        if self.channel.send(message).is_err() {
            std::process::exit(1);
        }
    }

    fn save(&self) {
        write_config(&self.config, &self.current_config);
    }

    fn gui(&mut self, ui: &mut Ui) {
        ui.ctx().set_pixels_per_point(self.display_scale);
        egui::Panel::left("sidebar")
            .resizable(false)
            .show_inside(ui, |ui| {
                ui.selectable_value(&mut self.current_tab, Tab::Aimbot, "Aimbot");
                ui.selectable_value(&mut self.current_tab, Tab::Player, "Player");
                ui.selectable_value(&mut self.current_tab, Tab::Hud, "Hud");
                ui.selectable_value(&mut self.current_tab, Tab::Grenades, "Grenades");
                ui.selectable_value(&mut self.current_tab, Tab::Unsafe, "Unsafe");
                ui.selectable_value(&mut self.current_tab, Tab::Config, "Config");
                ui.selectable_value(&mut self.current_tab, Tab::Application, "Application");

                ui.with_layout(egui::Layout::bottom_up(Align::Min), |ui| {
                    if ui.button("Report Issue").clicked() {
                        let _ = std::process::Command::new("xdg-open")
                            .arg("https://github.com/avitran0/deadlocked/issues")
                            .status();
                    }

                    ui.label(egui::RichText::new(format!("{}", self.game_status)).color(
                        match self.game_status {
                            GameStatus::Working => Colors::GREEN,
                            GameStatus::NotStarted => Colors::YELLOW,
                        },
                    ));

                    let frame_avg = if self.frame_times.is_empty() {
                        0.0f32
                    } else {
                        let frame_sum =
                            self.frame_times.iter().sum::<Duration>().as_secs_f32() * 1000.0;
                        frame_sum / self.frame_times.len() as f32
                    };
                    ui.label(format!("{frame_avg:.1} ms"));
                });
            });

        egui::CentralPanel::default().show_inside(ui, |ui| match self.current_tab {
            Tab::Aimbot => self.aimbot_settings(ui),
            Tab::Player => self.player_settings(ui),
            Tab::Hud => self.hud_settings(ui),
            Tab::Grenades => self.grenade_settings(ui),
            Tab::Unsafe => self.unsafe_settings(ui),
            Tab::Config => self.config_settings(ui),
            Tab::Application => self.application_settings(ui),
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

    pub fn render(&mut self) {
        let self_ptr = self as *mut Self;

        let gui = self.gui.as_mut().unwrap();

        if let Err(err) = gui.make_current() {
            utils::error!("could not make gui window current: {err}");
            return;
        }
        gui.run(|ui| (unsafe { &mut *self_ptr }).gui(ui));
        gui.clear();
        gui.paint();

        if let Err(err) = gui.swap_buffers() {
            utils::error!("could not swap gui window buffers: {err}");
            return;
        }

        let overlay = self.overlay.as_mut().unwrap();

        overlay.window().set_cursor_hittest(false).unwrap();
        if let Err(err) = overlay.make_current() {
            utils::error!("could not make overlay window current: {err}");
            return;
        }

        overlay.run(move |ui| {
            (unsafe { &mut *self_ptr }).overlay(ui);
        });
        overlay.clear();
        overlay.paint();

        if let Err(err) = overlay.swap_buffers() {
            utils::error!("could not swap overlay window buffers: {err}");
        }
    }
}
