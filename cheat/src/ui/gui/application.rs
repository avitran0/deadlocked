use egui::Ui;

use crate::{
    ui::{app::AppState, gui::helpers::open_url},
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

            match self.update_status.clone() {
                UpdateStatus::UpToDate => {
                    ui.colored_label(crate::ui::color::Colors::GREEN, "Up to date");
                }
                UpdateStatus::Available {
                    version,
                    html_url,
                    asset_url,
                } => {
                    ui.colored_label(
                        crate::ui::color::Colors::YELLOW,
                        format!("Update available: {version}"),
                    );
                    if ui.link("Release Notes").clicked() {
                        open_url(&html_url);
                    }
                    if let Some(asset_url) = &asset_url {
                        self.update_button(ui, asset_url);
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

    pub(crate) fn update_button(&mut self, ui: &mut Ui, asset_url: &str) {
        use crate::update::ApplyStatus;

        let running = matches!(*self.update_apply_status.lock(), ApplyStatus::Downloading);

        ui.add_enabled_ui(!running, |ui| {
            if ui.button("Update Now").clicked() {
                let status = self.update_apply_status.clone();
                let asset_url = asset_url.to_string();
                *status.lock() = ApplyStatus::Downloading;
                std::thread::spawn(move || {
                    let result = crate::update::apply_update(&asset_url);
                    *status.lock() = match result {
                        Ok(()) => ApplyStatus::Done,
                        Err(err) => ApplyStatus::Error(err),
                    };
                });
            }
        });

        match &*self.update_apply_status.lock() {
            ApplyStatus::Idle => {}
            ApplyStatus::Downloading => {
                ui.label("downloading...");
            }
            ApplyStatus::Done => {
                ui.colored_label(
                    crate::ui::color::Colors::GREEN,
                    "Updated. Restart deadlocked to use the new version.",
                );
            }
            ApplyStatus::Error(err) => {
                ui.colored_label(crate::ui::color::Colors::RED, err);
            }
        }
    }
}
