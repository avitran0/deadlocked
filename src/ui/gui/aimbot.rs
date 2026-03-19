use egui::{DragValue, Ui};
use strum::IntoEnumIterator as _;

use crate::{
    cs2::bones::Bones,
    ui::{
        app::App,
        drag_range::DragRange,
        gui::helpers::{
            checkbox, checkbox_hover, collapsing_open, combo_box, drag, keybind, scroll,
        },
    },
};

#[derive(PartialEq)]
pub enum AimbotTab {
    Global,
    Weapon,
}

impl App {
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
                self.send_config();
            }

            if self.aimbot_tab == AimbotTab::Weapon
                && checkbox_hover(
                    ui,
                    "Enable Override",
                    "Enable aimbot settings override for a specific weapon",
                    &mut self.weapon_config().aimbot.enable_override,
                )
            {
                self.send_config();
            }

            if checkbox(
                ui,
                "Enable Aimbot",
                &mut self.weapon_config().aimbot.enabled,
            ) {
                self.send_config();
            }

            if combo_box(
                ui,
                "aimbot_mode",
                "Mode",
                &mut self.weapon_config().aimbot.mode,
            ) {
                self.send_config();
            }
        });

        ui.collapsing("Targeting", |ui| {
            if checkbox_hover(
                ui,
                "Target Friendlies",
                "Only active in custom game modes (workshop/custom maps)",
                &mut self.weapon_config().aimbot.target_friendlies,
            ) {
                self.send_config();
            }

            if checkbox_hover(
                ui,
                "Distance-Adjusted FOV",
                "Adjusts FOV based on target distance",
                &mut self.weapon_config().aimbot.distance_adjusted_fov,
            ) {
                self.send_config();
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
                self.send_config();
            }

            if drag(
                ui,
                "Smooth",
                DragValue::new(&mut self.weapon_config().aimbot.smooth)
                    .range(0.0..=20.0)
                    .speed(0.02)
                    .max_decimals(1),
            ) {
                self.send_config();
            }

            if drag(
                ui,
                "Start Bullet",
                DragValue::new(&mut self.weapon_config().aimbot.start_bullet)
                    .range(0..=10)
                    .speed(0.05),
            ) {
                self.send_config();
            }

            if combo_box(
                ui,
                "targeting_mode",
                "Targeting Mode",
                &mut self.weapon_config().aimbot.targeting_mode,
            ) {
                self.send_config();
            }
        });

        ui.collapsing("Checks", |ui| {
            if checkbox(
                ui,
                "Visibility Check",
                &mut self.weapon_config().aimbot.visibility_check,
            ) {
                self.send_config();
            }

            if checkbox_hover(
                ui,
                "Smoke/Wall Check",
                "Blocks aiming through map geometry and smoke volumes",
                &mut self.weapon_config().aimbot.smoke_wall_check,
            ) {
                self.send_config();
            }

            if checkbox(
                ui,
                "Flash Check",
                &mut self.weapon_config().aimbot.flash_check,
            ) {
                self.send_config();
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
                    self.send_config();
                }
            }
        });
    }

    fn aimbot_right(&mut self, ui: &mut Ui) {
        collapsing_open(ui, "Triggerbot", |ui| {
            if self.aimbot_tab == AimbotTab::Weapon
                && ui
                    .checkbox(
                        &mut self.weapon_config().triggerbot.enable_override,
                        "Enable Override",
                    )
                    .changed()
            {
                self.send_config();
            }

            if ui
                .checkbox(
                    &mut self.weapon_config().triggerbot.enabled,
                    "Enable Triggerbot",
                )
                .changed()
            {
                self.send_config();
            }

            if keybind(
                ui,
                "triggerbot_hotkey",
                "Hotkey",
                &mut self.config.aim.triggerbot_hotkey,
            ) {
                self.send_config();
            }

            ui.horizontal(|ui| {
                if ui
                    .add(DragRange::new(
                        &mut self.weapon_config().triggerbot.delay,
                        0..=999,
                    ))
                    .changed()
                {
                    self.send_config();
                }
                ui.label("Delay (ms)");
            });

            if combo_box(
                ui,
                "triggerbot_mode",
                "Mode",
                &mut self.weapon_config().triggerbot.mode,
            ) {
                self.send_config();
            }

            if ui
                .checkbox(&mut self.weapon_config().triggerbot.head_only, "Head Only")
                .changed()
            {
                self.send_config();
            }

            ui.horizontal(|ui| {
                if ui
                    .add(
                        DragValue::new(&mut self.weapon_config().triggerbot.shot_duration)
                            .range(0..=2000)
                            .speed(10.0),
                    )
                    .changed()
                {
                    self.send_config();
                }
                ui.label("Additional Duration (ms)");
            });
        });

        ui.collapsing("Checks\u{200b}", |ui| {
            if ui
                .checkbox(
                    &mut self.weapon_config().triggerbot.flash_check,
                    "Flash Check",
                )
                .changed()
            {
                self.send_config();
            }

            if ui
                .checkbox(
                    &mut self.weapon_config().triggerbot.scope_check,
                    "Scope Check",
                )
                .changed()
            {
                self.send_config();
            }

            if ui
                .checkbox(
                    &mut self.weapon_config().triggerbot.velocity_check,
                    "Velocity Check",
                )
                .on_hover_text("Only shoot if the player moves slower than the specified threshold")
                .changed()
            {
                self.send_config();
            }

            ui.horizontal(|ui| {
                if ui
                    .add(
                        DragValue::new(&mut self.weapon_config().triggerbot.velocity_threshold)
                            .range(0..=5000),
                    )
                    .on_hover_text(
                        "Maximum velocity at which the triggerbot can shoot (in CS2 Units)",
                    )
                    .changed()
                {
                    self.send_config();
                }
                ui.label("Velocity Threshold");
            });
        });

        collapsing_open(ui, "RCS", |ui| {
            if self.aimbot_tab == AimbotTab::Weapon
                && ui
                    .checkbox(
                        &mut self.weapon_config().rcs.enable_override,
                        "Enable Override",
                    )
                    .changed()
            {
                self.send_config();
            }

            if ui
                .checkbox(&mut self.weapon_config().rcs.enabled, "Enable RCS")
                .changed()
            {
                self.send_config();
            }

            ui.horizontal(|ui| {
                if ui
                    .add(
                        DragValue::new(&mut self.weapon_config().rcs.smooth)
                            .range(0.0..=1.0)
                            .speed(0.02),
                    )
                    .on_hover_text("Lower values mean more direct recoil control")
                    .changed()
                {
                    self.send_config();
                }
                ui.label("RCS Smooth");
            });
        });
    }
}
