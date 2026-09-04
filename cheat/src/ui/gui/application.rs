use egui::Ui;

use crate::{
    config::application::write_app_config,
    ui::{
        app::AppState,
        gui::helpers::{checkbox, open_url},
    },
    update::UpdateStatus,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");

impl AppState {
    pub fn application_settings(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.heading("deadlocked");
            ui.label("author: avitrano");
            ui.label(format!("Version: v{VERSION}"));

            ui.separator();

            if checkbox(
                ui,
                "Check for Updates",
                &mut self.app_config.check_for_updates,
            ) {
                write_app_config(&self.app_config);
                if !self.app_config.check_for_updates {
                    self.update_status = UpdateStatus::UpToDate;
                    self.update_popup = false;
                }
            }

            ui.separator();

            if !self.app_config.check_for_updates {
                ui.colored_label(
                    crate::ui::color::Colors::YELLOW,
                    "Update checks disabled",
                );
            } else {
                match &self.update_status {
                    UpdateStatus::UpToDate => {
                        ui.colored_label(crate::ui::color::Colors::GREEN, "Up to date");
                    }
                    UpdateStatus::Available { version, url } => {
                        ui.colored_label(
                            crate::ui::color::Colors::YELLOW,
                            format!("Update available: {version}"),
                        );
                        if ui.link("Download").clicked() {
                            open_url(url);
                        }
                    }
                    UpdateStatus::Error(err) => {
                        ui.colored_label(
                            crate::ui::color::Colors::RED,
                            format!("Update check failed: {err}"),
                        );
                    }
                }
            }
        });
    }
}
