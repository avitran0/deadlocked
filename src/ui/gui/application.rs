use egui::Ui;

use crate::{ui::app::App, update::UpdateStatus};

const VERSION: &str = env!("CARGO_PKG_VERSION");

impl App {
    pub fn application_settings(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.heading("deadlocked");
            ui.label("author: avitrano");
            ui.label(format!("Version: v{VERSION}"));

            ui.separator();

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
                        let _ = std::process::Command::new("xdg-open").arg(url).status();
                    }
                }
                UpdateStatus::Error(err) => {
                    ui.colored_label(
                        crate::ui::color::Colors::RED,
                        format!("Update check failed: {err}"),
                    );
                }
            }
        });
    }
}
