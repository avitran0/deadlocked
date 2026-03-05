use egui::{DragValue, Ui};
use strum::IntoEnumIterator as _;

use crate::{
    config::KeyMode,
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

            let aimbot_mode = self.weapon_config().aimbot.mode.clone();
            if aimbot_mode != KeyMode::Shoot {
                if keybind(
                    ui,
                    "aimbot_hotkey",
                    "Hotkey",
                    &mut self.config.aim.aimbot_hotkey,
                ) {
                    self.send_config();
                }
            } else {
                ui.label("Hotkey: not used in Shoot mode");
            }

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

            ui.horizontal(|ui| {
                if ui
                    .add(
                        DragValue::new(&mut self.weapon_config().aimbot.start_bullet)
                            .range(0..=10)
                            .speed(0.05),
                    )
                    .on_hover_text(
                        "How many bullets you need to shoot\nbefore the aimbot starts aiming",
                    )
                    .changed()
                {
                    self.send_config();
                }
                ui.label("Start Bullet");
            });

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
            if ui
                .checkbox(
                    &mut self.weapon_config().aimbot.visibility_check,
                    "Visibility Check",
                )
                .changed()
            {
                self.send_config();
            }

            if ui
                .checkbox(
                    &mut self.weapon_config().aimbot.smoke_wall_check,
                    "Smoke/Wall Check",
                )
                .on_hover_text(
                    "Requires parsed map BVH, clear geometry LOS, and no smoke between you and target",
                )
                .changed()
            {
                self.send_config();
            }

            if ui
                .checkbox(&mut self.weapon_config().aimbot.flash_check, "Flash Check")
                .changed()
            {
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

            if combo_box(
                ui,
                "triggerbot_mode",
                "Mode",
                &mut self.weapon_config().triggerbot.mode,
            ) {
                self.send_config();
            }

            let trigger_mode = self.weapon_config().triggerbot.mode.clone();
            if trigger_mode != KeyMode::Shoot {
                if keybind(
                    ui,
                    "triggerbot_hotkey",
                    "Hotkey",
                    &mut self.config.aim.triggerbot_hotkey,
                ) {
                    self.send_config();
                }
            } else {
                ui.label("Hotkey: not used in Shoot mode");
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
                ui.label("Press Duration (ms)");
            });

            if ui
                .checkbox(
                    &mut self.weapon_config().triggerbot.aim_assist,
                    "Trigger Aim Assist",
                )
                .on_hover_text("Micro-adjusts aim before firing to reduce misses")
                .changed()
            {
                self.send_config();
            }

            ui.horizontal(|ui| {
                if ui
                    .add(
                        DragValue::new(&mut self.weapon_config().triggerbot.aim_fov)
                            .range(0.1..=30.0)
                            .suffix("°")
                            .speed(0.02)
                            .max_decimals(2),
                    )
                    .changed()
                {
                    self.send_config();
                }
                ui.label("Assist FOV");
            });

            ui.horizontal(|ui| {
                if ui
                    .add(
                        DragValue::new(&mut self.weapon_config().triggerbot.fire_fov)
                            .range(0.05..=10.0)
                            .suffix("°")
                            .speed(0.01)
                            .max_decimals(2),
                    )
                    .on_hover_text("How close aim must be before triggerbot clicks")
                    .changed()
                {
                    self.send_config();
                }
                ui.label("Fire FOV");
            });

            ui.horizontal(|ui| {
                if ui
                    .add(
                        DragValue::new(&mut self.weapon_config().triggerbot.aim_smooth)
                            .range(0.0..=20.0)
                            .speed(0.05)
                            .max_decimals(2),
                    )
                    .on_hover_text("Lower values aim faster during trigger adjustment")
                    .changed()
                {
                    self.send_config();
                }
                ui.label("Assist Smooth");
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
                    &mut self.weapon_config().triggerbot.smoke_wall_check,
                    "Smoke/Wall Check",
                )
                .on_hover_text(
                    "Requires parsed map BVH, clear geometry LOS, and no smoke between you and target",
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
                .on_hover_text("Only shoot if YOUR horizontal movement speed is below threshold")
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
                        "Maximum horizontal speed at which triggerbot can shoot (CS2 units/sec)",
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
