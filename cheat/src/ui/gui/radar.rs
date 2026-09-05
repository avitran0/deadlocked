use egui::{Button, OpenUrl, Ui};
use uuid::Uuid;

use crate::{
    config::application::write_app_config,
    message::RadarStatus,
    ui::{
        app::AppState,
        color::Colors,
        gui::helpers::{checkbox, collapsing_open, scroll},
    },
};

impl AppState {
    pub fn radar_settings(&mut self, ui: &mut Ui) {
        scroll(ui, "radar_settings", |ui| self.radar_settings_content(ui));
    }

    fn radar_settings_content(&mut self, ui: &mut Ui) {
        collapsing_open(ui, "Radar", |ui| {
            ui.horizontal(|ui| {
                if checkbox(ui, "Enabled", &mut self.config.radar.enabled) {
                    self.send_config_radar();
                }

                let (color, text) = match self.radar_status {
                    RadarStatus::FailedToConnect => (Colors::RED, "Failed to connect"),
                    RadarStatus::Disabled => (Colors::YELLOW, "Disabled"),
                    RadarStatus::Disconnected => (Colors::YELLOW, "Disconnected"),
                    RadarStatus::Connected => (Colors::GREEN, "Connected"),
                };
                ui.colored_label(color, text);
            });
        });

        collapsing_open(ui, "Connection", |ui| {
            ui.label("Server host (without protocol or port):");
            if ui
                .text_edit_singleline(&mut self.config.radar.url)
                .changed()
            {
                self.send_config_radar();
            }
        });

        collapsing_open(ui, "Session", |ui| {
            ui.horizontal(|ui| {
                ui.label("Session UUID:");
                ui.monospace(self.app_config.radar_uuid.to_string());
            });

            let link = format!(
                "https://radar.avitrano.com/?url={}&game={}",
                urlencoding::encode(&self.config.radar.url),
                self.app_config.radar_uuid
            );
            let connected = matches!(self.radar_status, RadarStatus::Connected);

            ui.horizontal(|ui| {
                if ui.add_enabled(connected, Button::new("Open")).clicked() {
                    ui.ctx().open_url(OpenUrl::new_tab(link.clone()));
                }
                if ui.button("Copy link").clicked() {
                    ui.ctx().copy_text(link);
                }
            });

            if ui
                .button("Reset UUID")
                .on_hover_text("Disconnect and create a new radar session")
                .clicked()
            {
                self.app_config.radar_uuid = Uuid::new_v4();
                write_app_config(&self.app_config);
                self.radar_status = RadarStatus::Disconnected;
                self.send_config_radar();
            }
        });
    }
}
