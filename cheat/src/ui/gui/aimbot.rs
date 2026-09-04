use egui::{DragValue, Ui};
use shared::Bones;
use strum::IntoEnumIterator as _;

use crate::{
    config::aim::{AimbotConfig, RcsConfig, TriggerbotConfig},
    ui::{
        app::AppState,
        drag_range::DragRange,
        gui::helpers::{
            checkbox, checkbox_hover, collapsing_open, combo_box, drag, keybind, reset_button,
            scroll,
        },
    },
};

#[derive(PartialEq)]
pub enum AimbotTab {
    Global,
    Weapon,
}

impl AppState {
    pub fn aimbot_settings(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.aimbot_tab, AimbotTab::Global, "Global");
            ui.selectable_value(&mut self.aimbot_tab, AimbotTab::Weapon, "Weapon");
            if self.aimbot_tab == AimbotTab::Weapon {
                combo_box(ui, "aimbot_weapon", "Weapon", &mut self.aimbot_weapon);
            }
        });
        ui.separator();
        ui.columns(2, |cols| {
            let left = &mut cols[0];
            scroll(left, "aimbot_left", |ui| self.aimbot_left(ui));

            let right = &mut cols[1];
            scroll(right, "aimbot_right", |ui| self.aimbot_right(ui));
        });
    }

    fn aimbot_left(&mut self, ui: &mut Ui) {
        collapsing_open(ui, "Aimbot", |ui| {
            if keybind(
                ui,
                "aimbot_hotkey",
                "Hotkey",
                &mut self.config.aim.aimbot_hotkey,
            ) {
                self.send_config_game();
            }

            if self.aimbot_tab == AimbotTab::Weapon
                && checkbox_hover(
                    ui,
                    "Enable Override",
                    "Enable aimbot settings override for a specific weapon",
                    &mut self.weapon_config().aimbot.enable_override,
                )
            {
                self.send_config_game();
            }

            if checkbox(
                ui,
                "Enable Aimbot",
                &mut self.weapon_config().aimbot.enabled,
            ) {
                self.send_config_game();
            }

            if combo_box(
                ui,
                "aimbot_mode",
                "Mode",
                &mut self.weapon_config().aimbot.mode,
            ) {
                self.send_config_game();
            }

            if reset_button(ui) {
                let override_enabled = self.weapon_config().aimbot.enable_override;
                self.weapon_config().aimbot = AimbotConfig {
                    enable_override: override_enabled,
                    ..AimbotConfig::default()
                };
                self.config.aim.aimbot_hotkey = crate::config::aim::AimConfig::default().aimbot_hotkey;
                self.send_config_game();
            }
        });

        ui.collapsing("Targeting", |ui| {
            if checkbox(
                ui,
                "Target Friendlies",
                &mut self.weapon_config().aimbot.target_friendlies,
            ) {
                self.send_config_game();
            }

            if checkbox_hover(
                ui,
                "Distance-Adjusted FOV",
                "Adjusts FOV based on target distance",
                &mut self.weapon_config().aimbot.distance_adjusted_fov,
            ) {
                self.send_config_game();
            }

            if drag(
                ui,
                "FOV",
                DragValue::new(&mut self.weapon_config().aimbot.fov)
                    .range(0.1..=360.0)
                    .suffix("°")
                    .speed(0.02)
                    .max_decimals(1),
            ) {
                self.send_config_game();
            }

            if drag(
                ui,
                "Smooth",
                DragValue::new(&mut self.weapon_config().aimbot.smooth)
                    .range(0.0..=20.0)
                    .speed(0.02)
                    .max_decimals(1),
            ) {
                self.send_config_game();
            }

            if drag(
                ui,
                "Inertia",
                DragValue::new(&mut self.weapon_config().aimbot.inertia)
                    .range(0.0..=1.0)
                    .speed(0.005)
                    .max_decimals(2),
            ) {
                self.send_config_game();
            }

            if drag(
                ui,
                "Prediction",
                DragValue::new(&mut self.weapon_config().aimbot.prediction_time)
                    .range(0.0..=0.25)
                    .suffix(" s")
                    .speed(0.002)
                    .max_decimals(2),
            ) {
                self.send_config_game();
            }

            if drag(
                ui,
                "Start Bullet",
                DragValue::new(&mut self.weapon_config().aimbot.start_bullet)
                    .range(0..=10)
                    .speed(0.05),
            ) {
                self.send_config_game();
            }

            if combo_box(
                ui,
                "targeting_mode",
                "Targeting Mode",
                &mut self.weapon_config().aimbot.targeting_mode,
            ) {
                self.send_config_game();
            }

            if reset_button(ui) {
                let defaults = AimbotConfig::default();
                let aimbot = &mut self.weapon_config().aimbot;
                aimbot.target_friendlies = defaults.target_friendlies;
                aimbot.distance_adjusted_fov = defaults.distance_adjusted_fov;
                aimbot.fov = defaults.fov;
                aimbot.smooth = defaults.smooth;
                aimbot.inertia = defaults.inertia;
                aimbot.prediction_time = defaults.prediction_time;
                aimbot.start_bullet = defaults.start_bullet;
                aimbot.targeting_mode = defaults.targeting_mode;
                self.send_config_game();
            }
        });

        ui.collapsing("Checks", |ui| {
            if checkbox(
                ui,
                "Visibility Check",
                &mut self.weapon_config().aimbot.visibility_check,
            ) {
                self.send_config_game();
            }

            if checkbox(
                ui,
                "Flash Check",
                &mut self.weapon_config().aimbot.flash_check,
            ) {
                self.send_config_game();
            }

            if reset_button(ui) {
                let defaults = AimbotConfig::default();
                let aimbot = &mut self.weapon_config().aimbot;
                aimbot.visibility_check = defaults.visibility_check;
                aimbot.flash_check = defaults.flash_check;
                self.send_config_game();
            }
        });

        ui.collapsing("Bones", |ui| {
            for bone in Bones::iter() {
                let text = format!("{:?}", bone);
                let index = self
                    .weapon_config()
                    .aimbot
                    .bones
                    .iter()
                    .position(|b| *b == bone);
                if ui.selectable_label(index.is_some(), text).clicked() {
                    if let Some(index) = index {
                        self.weapon_config().aimbot.bones.remove(index);
                    } else {
                        self.weapon_config().aimbot.bones.push(bone);
                    }
                    self.send_config_game();
                }
            }

            if reset_button(ui) {
                self.weapon_config().aimbot.bones = AimbotConfig::default().bones;
                self.send_config_game();
            }
        });
    }

    fn aimbot_right(&mut self, ui: &mut Ui) {
        collapsing_open(ui, "Triggerbot", |ui| {
            if self.aimbot_tab == AimbotTab::Weapon
                && checkbox(
                    ui,
                    "Enable Override",
                    &mut self.weapon_config().triggerbot.enable_override,
                )
            {
                self.send_config_game();
            }

            if checkbox(
                ui,
                "Enable Triggerbot",
                &mut self.weapon_config().triggerbot.enabled,
            ) {
                self.send_config_game();
            }

            if keybind(
                ui,
                "triggerbot_hotkey",
                "Hotkey",
                &mut self.config.aim.triggerbot_hotkey,
            ) {
                self.send_config_game();
            }

            if ui
                .add(DragRange::new(
                    "Delay (ms)",
                    &mut self.weapon_config().triggerbot.delay,
                    0..=999,
                ))
                .changed()
            {
                self.send_config_game();
            }

            if combo_box(
                ui,
                "triggerbot_mode",
                "Mode",
                &mut self.weapon_config().triggerbot.mode,
            ) {
                self.send_config_game();
            }

            if checkbox(
                ui,
                "Head Only",
                &mut self.weapon_config().triggerbot.head_only,
            ) {
                self.send_config_game();
            }

            if drag(
                ui,
                "Hold Duration (ms)",
                DragValue::new(&mut self.weapon_config().triggerbot.shot_duration)
                    .range(0..=2000)
                    .speed(10.0),
            ) {
                self.send_config_game();
            }

            if reset_button(ui) {
                let override_enabled = self.weapon_config().triggerbot.enable_override;
                self.weapon_config().triggerbot = TriggerbotConfig {
                    enable_override: override_enabled,
                    ..TriggerbotConfig::default()
                };
                self.config.aim.triggerbot_hotkey =
                    crate::config::aim::AimConfig::default().triggerbot_hotkey;
                self.send_config_game();
            }
        });

        ui.collapsing("Checks\u{200b}", |ui| {
            if checkbox(
                ui,
                "Flash Check",
                &mut self.weapon_config().triggerbot.flash_check,
            ) {
                self.send_config_game();
            }

            if checkbox(
                ui,
                "Scope Check",
                &mut self.weapon_config().triggerbot.scope_check,
            ) {
                self.send_config_game();
            }

            if checkbox_hover(
                ui,
                "Velocity Check",
                "Only shoot if YOU move slower than the threshold",
                &mut self.weapon_config().triggerbot.velocity_check,
            ) {
                self.send_config_game();
            }

            if drag(
                ui,
                "Velocity Threshold",
                DragValue::new(&mut self.weapon_config().triggerbot.velocity_threshold)
                    .range(0..=5000),
            ) {
                self.send_config_game();
            }

            if reset_button(ui) {
                let defaults = TriggerbotConfig::default();
                let trigger = &mut self.weapon_config().triggerbot;
                trigger.flash_check = defaults.flash_check;
                trigger.scope_check = defaults.scope_check;
                trigger.velocity_check = defaults.velocity_check;
                trigger.velocity_threshold = defaults.velocity_threshold;
                self.send_config_game();
            }
        });

        collapsing_open(ui, "RCS", |ui| {
            if self.aimbot_tab == AimbotTab::Weapon
                && checkbox(
                    ui,
                    "Enable Override",
                    &mut self.weapon_config().rcs.enable_override,
                )
            {
                self.send_config_game();
            }

            if checkbox(ui, "Enable RCS", &mut self.weapon_config().rcs.enabled) {
                self.send_config_game();
            }

            if ui
                .horizontal(|ui| {
                    let rcs = &mut self.weapon_config().rcs;
                    let x = ui.add(
                        DragValue::new(&mut rcs.strength.x)
                            .prefix("X: ")
                            .range(0.0..=1.0)
                            .speed(0.01),
                    );
                    let y = ui.add(
                        DragValue::new(&mut rcs.strength.y)
                            .prefix("Y: ")
                            .range(0.0..=1.0)
                            .speed(0.01),
                    );
                    ui.label("Strength");
                    (x | y).changed()
                })
                .inner
            {
                self.send_config_game();
            }

            if reset_button(ui) {
                let override_enabled = self.weapon_config().rcs.enable_override;
                self.weapon_config().rcs = RcsConfig {
                    enable_override: override_enabled,
                    ..RcsConfig::default()
                };
                self.send_config_game();
            }
        });
    }
}
