use egui::{OpenUrl, Ui};

use crate::{
    message::RadarStatus,
    ui::{app::AppState, color::Colors, gui::helpers::checkbox},
};

impl AppState {
    pub fn radar_settings(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if checkbox(ui, "Enabled", &mut self.config.radar.enabled) {
                self.send_config_radar();
            }

            let (color, text) = match self.radar_status {
                RadarStatus::FailedToConnect => (Colors::RED, "Failed to connect"),
                RadarStatus::Disabled => (Colors::YELLOW, "Disabled"),
                RadarStatus::Disconnected => (Colors::YELLOW, "Disconnected"),
                RadarStatus::Connected(_) => (Colors::GREEN, "Connected"),
            };
            ui.colored_label(color, text);

            if let RadarStatus::Connected(uuid) = &self.radar_status
                && ui.button("Open").clicked()
            {
                ui.ctx().open_url(OpenUrl::new_tab(format!(
                    "https://radar.avitrano.com/?url={}&game={uuid}",
                    urlencoding::encode(&self.config.radar.url)
                )));
            }
        });

        ui.label("Server host (without protocol or port):");
        if ui
            .text_edit_singleline(&mut self.config.radar.url)
            .changed()
        {
            self.send_config_radar();
        }
    }
}
