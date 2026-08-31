use std::time::Duration;

use egui::{Align, Ui};
use shared::Data;

use crate::{
    config::{aim::WeaponConfig, write_config},
    message::{GameMessage, GameStatus},
    ui::{
        app::{App, AppState},
        color::Colors,
        gui::{
            aimbot::AimbotTab,
            helpers::{open_url, text_settings_popup},
        },
        window_context::WindowContext,
    },
    update::UpdateStatus,
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

impl AppState {
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
            .show(ui, |ui| {
                ui.selectable_value(&mut self.current_tab, Tab::Aimbot, "Aimbot");
                ui.selectable_value(&mut self.current_tab, Tab::Player, "Player");
                ui.selectable_value(&mut self.current_tab, Tab::Hud, "Hud");
                ui.selectable_value(&mut self.current_tab, Tab::Grenades, "Grenades");
                ui.selectable_value(&mut self.current_tab, Tab::Unsafe, "Unsafe");
                ui.selectable_value(&mut self.current_tab, Tab::Config, "Config");
                ui.selectable_value(&mut self.current_tab, Tab::Application, "Application");

                ui.with_layout(egui::Layout::bottom_up(Align::Min), |ui| {
                    ui.label(concat!("v", env!("CARGO_PKG_VERSION")));

                    if ui.button("Report Issue").clicked() {
                        open_url("https://github.com/avitran0/deadlocked/issues");
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

        egui::CentralPanel::default().show(ui, |ui| match self.current_tab {
            Tab::Aimbot => self.aimbot_settings(ui),
            Tab::Player => self.player_settings(ui),
            Tab::Hud => self.hud_settings(ui),
            Tab::Grenades => self.grenade_settings(ui),
            Tab::Unsafe => self.unsafe_settings(ui),
            Tab::Config => self.config_settings(ui),
            Tab::Application => self.application_settings(ui),
        });

        self.render_text_popups(ui);

        if self.update_popup {
            let mut close = false;
            egui::Window::new("Update Available")
                .id(egui::Id::new("update_popup"))
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ui.ctx(), |ui| {
                    if let UpdateStatus::Available { version, url } = &self.update_status {
                        ui.label(
                            egui::RichText::new(format!("Update {version} available!"))
                                .color(Colors::YELLOW)
                                .size(18.0),
                        );
                        ui.separator();
                        ui.label("A new version of deadlocked is ready to download.");
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            if ui.button("Download").clicked() {
                                open_url(url);
                            }
                            if ui.button("Dismiss").clicked() {
                                close = true;
                            }
                        });
                    }
                });
            if close {
                self.update_popup = false;
            }
        }
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

    fn render_text_popups(&mut self, ui: &mut Ui) {
        let text = &mut self.config.hud.overlay_text;
        let mut changed = false;
        changed |= text_settings_popup(
            ui,
            "Status Text",
            &mut text.status_text,
            &mut self.text_popup,
            "status_text",
        );
        changed |= text_settings_popup(
            ui,
            "Player Name",
            &mut text.player_name,
            &mut self.text_popup,
            "player_name",
        );
        changed |= text_settings_popup(
            ui,
            "Player Tags",
            &mut text.player_tags,
            &mut self.text_popup,
            "player_tags",
        );
        changed |= text_settings_popup(
            ui,
            "Weapon Icon",
            &mut text.weapon_icon,
            &mut self.text_popup,
            "weapon_icon",
        );
        changed |= text_settings_popup(
            ui,
            "Ammo",
            &mut text.ammo_text,
            &mut self.text_popup,
            "ammo_text",
        );
        changed |= text_settings_popup(
            ui,
            "Weapon Name",
            &mut text.weapon_name,
            &mut self.text_popup,
            "weapon_name",
        );
        changed |= text_settings_popup(
            ui,
            "Bomb Timer",
            &mut text.bomb_timer,
            &mut self.text_popup,
            "bomb_timer",
        );
        changed |= text_settings_popup(
            ui,
            "Grenade Name",
            &mut text.grenade_name,
            &mut self.text_popup,
            "grenade_name",
        );
        changed |= text_settings_popup(
            ui,
            "Grenade Lineup",
            &mut text.grenade_lineup,
            &mut self.text_popup,
            "grenade_lineup",
        );
        changed |= text_settings_popup(
            ui,
            "Keybind List",
            &mut text.keybind_list,
            &mut self.text_popup,
            "keybind_list",
        );
        changed |= text_settings_popup(
            ui,
            "Spectator List",
            &mut text.spectator_list,
            &mut self.text_popup,
            "spectator_list",
        );
        if changed {
            self.send_config();
        }
    }
}

impl App {
    pub fn render(&mut self) {
        let gui = self.gui.as_mut().unwrap();
        let overlay = self.overlay.as_mut().unwrap();
        let state = &mut self.state;

        if let Err(err) = gui.make_current() {
            utils::error!("could not make gui window current: {err}");
            return;
        }
        gui.run(|ui| state.gui(ui));
        gui.clear();
        gui.paint();

        if let Err(err) = gui.swap_buffers() {
            utils::error!("could not swap gui window buffers: {err}");
            return;
        }

        overlay.window().set_cursor_hittest(false).unwrap();
        {
            let data_guard = state.data.lock();
            Self::update_overlay_window(overlay, &data_guard);
        }
        if let Err(err) = overlay.make_current() {
            utils::error!("could not make overlay window current: {err}");
            return;
        }

        overlay.run(move |ui| state.overlay(ui));
        overlay.clear();
        overlay.paint();

        if let Err(err) = overlay.swap_buffers() {
            utils::error!("could not swap overlay window buffers: {err}");
        }
    }

    fn update_overlay_window(overlay: &WindowContext, data: &Data) {
        use winit::dpi::PhysicalPosition;
        let position =
            PhysicalPosition::new(data.window_position.x as i32, data.window_position.y as i32);
        if !match overlay.window().outer_position() {
            Ok(pos) => pos == position,
            Err(_) => false,
        } {
            overlay.window().set_outer_position(position);
        }

        let size = winit::dpi::PhysicalSize::new(
            data.window_size.x.max(1.0) as u32,
            data.window_size.y.max(1.0) as u32,
        );
        if overlay.window().inner_size() != size {
            let _ = overlay.window().request_inner_size(size);
        }
    }
}
