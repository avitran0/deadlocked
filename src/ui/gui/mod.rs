use std::time::Instant;
use egui::{Align, Align2, CollapsingHeader, Color32, Context, DragValue, RichText, Ui};

use crate::{
    config::{BASE_PATH, VERSION, WeaponConfig, write_config},
    message::{Envelope, GameStatus, Message, Target},
    os::{
        crash::report_error,
        mouse::{DeviceStatus, discover_mice},
    },
    ui::{app::App, color::Colors, gui::aimbot::AimbotTab},
};

pub mod aimbot;
mod config;
mod grenade;
mod hud;
mod player;
mod radar;
mod r#unsafe;

#[derive(PartialEq)]
pub enum Tab {
    Aimbot,
    Player,
    Hud,
    Radar,
    Grenades,
    Unsafe,
    Config,
}

impl App {
    pub fn send_config(&self) {
        self.send_message(Message::Config(Box::new(self.config.clone())), Target::Game);
        self.save();
    }

    pub fn send_message(&self, message: Message, target: Target) {
        if self.tx.send(Envelope { target, message }).is_err() {
            std::process::exit(1);
        }
    }

    fn save(&self) {
        write_config(&self.config, &self.current_config);
    }

    fn gui(&mut self, ctx: &Context) {
        ctx.set_pixels_per_point(self.display_scale);
        egui::SidePanel::left("sidebar")
            .resizable(false)
            .show(ctx, |ui| {
                ui.selectable_value(&mut self.current_tab, Tab::Aimbot, "\u{f04fe} Aimbot");
                ui.selectable_value(&mut self.current_tab, Tab::Player, "\u{f0013} Player");
                ui.selectable_value(&mut self.current_tab, Tab::Hud, "\u{f0379} Hud");
                ui.selectable_value(&mut self.current_tab, Tab::Radar, "\u{f0437} Radar");
                ui.selectable_value(&mut self.current_tab, Tab::Grenades, "\u{f0691} Grenades");
                ui.selectable_value(&mut self.current_tab, Tab::Unsafe, "\u{f0ce6} Unsafe");
                ui.selectable_value(&mut self.current_tab, Tab::Config, "\u{f168b} Config");

                ui.with_layout(egui::Layout::bottom_up(Align::Min), |ui| {
                    if ui.button("Report Issue").clicked() {
                        std::process::Command::new("xdg-open")
                            .arg("https://github.com/avitran0/deadlocked/issues")
                            .status()
                            .unwrap();
                    }
                    if ui.button("Config Folder").clicked() {
                        std::process::Command::new("xdg-open")
                            .arg(BASE_PATH.as_os_str())
                            .status()
                            .unwrap();
                    }
                    ui.label(VERSION);
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.add_game_status(ui);
            ui.separator();

            match self.current_tab {
                Tab::Aimbot => self.aimbot_settings(ui),
                Tab::Player => self.player_settings(ui),
                Tab::Hud => self.hud_settings(ui),
                Tab::Radar => self.radar_settings(ui),
                Tab::Grenades => self.grenade_settings(ui),
                Tab::Unsafe => self.unsafe_settings(ui),
                Tab::Config => self.config_settings(ui, ctx),
            }
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

    fn add_game_status(&mut self, ui: &mut Ui) {
        // logo in the top right
        ui.horizontal_top(|ui| {
            // Left side: original status + mouse combobox
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!("{}", self.game_status))
                        .line_height(Some(8.0))
                        .color(match self.game_status {
                            GameStatus::Working => Colors::GREEN,
                            GameStatus::GameNotStarted => Colors::YELLOW,
                        }),
                );

                let mouse_text = match &self.mouse_status {
                    DeviceStatus::Working(name) => name,
                    DeviceStatus::PermissionsRequired => {
                        "mouse input only works when user is in input group"
                    }
                    DeviceStatus::Disconnected => "mouse was disconnected",
                    DeviceStatus::NotFound => "no mouse was found",
                };

                let color = match &self.mouse_status {
                    DeviceStatus::Working(_) => Colors::SUBTEXT,
                    _ => Colors::YELLOW,
                };
                ui.label(
                    RichText::new(mouse_text)
                        .line_height(Some(8.0))
                        .color(color),
                );

                egui::ComboBox::new("mouse_device", "")
                    .selected_text(
                        self.selected_mouse
                            .as_deref()
                            .unwrap_or("No device selected"),
                    )
                    .show_ui(ui, |ui| {
                        for device in discover_mice() {
                            let label = format!("{} ({})", device.name, device.event_name);
                            if ui
                                .selectable_label(
                                    self.selected_mouse.as_deref() == Some(&device.event_name),
                                    &label,
                                )
                                .clicked()
                            {
                                self.selected_mouse = Some(device.event_name.clone());

                                self.send_message(
                                    Message::SelectMouse(device.event_name.clone()),
                                    Target::Game,
                                );
                            }
                        }
                    });
            });

            // Right-aligned logo: use a right_to_left layout for robust anchoring inside the central panel.
            ui.with_layout(egui::Layout::right_to_left(Align::Min), |ui| {
               
                let mut logo_size = egui::vec2(260.0, 28.0);
                let avail = ui.available_width();
                if avail < logo_size.x + 8.0 {
                    logo_size.x = (avail - 8.0).max(40.0); // minimum width
                }

                // Reserve space and draw the logo background + text inside the reserved rect
                let resp = ui.add_sized(logo_size, egui::Label::new(RichText::new("")));
                let rect = resp.rect;
                let painter = ui.painter();

                // background rect (darker accent)
                let accent = self.config.accent_color;
                let [ar, ag, ab, _] = accent.to_array();
                let darker_accent = Color32::from_rgb(
                    ((ar as u16 * 200) / 255) as u8,
                    ((ag as u16 * 200) / 255) as u8,
                    ((ab as u16 * 200) / 255) as u8,
                );
                painter.rect_filled(rect.shrink(2.0), 6.0, darker_accent);

                // compute per-substring colors:
                // split by characters; if manual split is enabled use logo_split_index; otherwise auto-split ceil(n/2)
                let text: String = self.logo_text.clone();
                let chars: Vec<char> = text.chars().collect();
                let len = chars.len();
                let left_count = if self.logo_manual_split {
                    (self.logo_split_index.max(0) as usize).min(len)
                } else {
                    (len + 1) / 2
                };

                let left_str: String = chars.iter().take(left_count).collect();
                let right_str: String = chars.iter().skip(left_count).collect();

                // when animation is ON: left blends A->B, right blends B->A
                let (color_left, color_right) = if self.logo_animated {
                    let elapsed = Instant::now().duration_since(self.logo_animation_start).as_secs_f32();
                    let phase = ((elapsed * self.logo_animation_speed * std::f32::consts::TAU).sin() * 0.5) + 0.5;
                    let [a_r, a_g, a_b, a_a] = self.logo_color_a.to_array();
                    let [b_r, b_g, b_b, b_a] = self.logo_color_b.to_array();
                    let lerp_byte = |a: u8, b: u8, t: f32| -> u8 {
                        let v = (a as f32) * (1.0 - t) + (b as f32) * t;
                        v.clamp(0.0, 255.0) as u8
                    };
                    let left = Color32::from_rgba_premultiplied(
                        lerp_byte(a_r, b_r, phase),
                        lerp_byte(a_g, b_g, phase),
                        lerp_byte(a_b, b_b, phase),
                        lerp_byte(a_a, b_a, phase),
                    );
                    let right = Color32::from_rgba_premultiplied(
                        lerp_byte(b_r, a_r, phase),
                        lerp_byte(b_g, a_g, phase),
                        lerp_byte(b_b, a_b, phase),
                        lerp_byte(b_a, a_a, phase),
                    );
                    (left, right)
                } else {
                    (self.logo_color_a, self.logo_color_b)
                };

                // Draw the full word as contiguous text but paint the left & right substrings separately.
                // Use an approximatio to measure width per substring.
                fn approx_text_width(font_size: f32, text: &str) -> f32 {
                    let avg_factor = 0.58_f32;
                    (text.len() as f32) * font_size * avg_factor
                }

                let font_size = 16.0;
                let font = egui::FontId::proportional(font_size);
                let w_left = approx_text_width(font_size, &left_str);
                let w_right = approx_text_width(font_size, &right_str);
                let total_w = w_left + w_right;
                let center = rect.center();

                // left substring center x
                let left_center_x = center.x - (total_w / 2.0) + (w_left / 2.0);
                let right_center_x = left_center_x + (w_left / 2.0) + (w_right / 2.0);

                let left_pos = egui::pos2(left_center_x, center.y);
                let right_pos = egui::pos2(right_center_x, center.y);

                // Outline offsets for legibility
                let outline_offsets = [
                    egui::vec2(-1.0, -1.0),
                    egui::vec2(1.0, -1.0),
                    egui::vec2(-1.0, 1.0),
                    egui::vec2(1.0, 1.0),
                ];

                for offs in outline_offsets {
                    painter.text(left_pos + offs, Align2::CENTER_CENTER, &left_str, font.clone(), Color32::BLACK);
                    painter.text(right_pos + offs, Align2::CENTER_CENTER, &right_str, font.clone(), Color32::BLACK);
                }
                painter.text(left_pos, Align2::CENTER_CENTER, &left_str, font.clone(), color_left);
                painter.text(right_pos, Align2::CENTER_CENTER, &right_str, font.clone(), color_right);

                // hover cursor change
                if resp.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
            });
        });
    }

    fn color_picker(&self, ui: &mut Ui, color: &Color32, label: &str) -> Option<Color32> {
        let [mut r, mut g, mut b, _] = color.to_array();
        let res = ui
            .horizontal(|ui| {
                let (response, painter) =
                    ui.allocate_painter(ui.spacing().interact_size, egui::Sense::hover());
                painter.rect_filled(
                    response.rect,
                    ui.style().visuals.widgets.inactive.corner_radius,
                    *color,
                );
                let mut res = ui.add(DragValue::new(&mut r).prefix("r: "));
                res = res.union(ui.add(DragValue::new(&mut g).prefix("g: ")));
                res = res.union(ui.add(DragValue::new(&mut b).prefix("b: ")));
                ui.label(label);
                res
            })
            .inner;

        if res.changed() {
            Some(Color32::from_rgb(r, g, b))
        } else {
            None
        }
    }

    pub fn render(&mut self) {
        let self_ptr = self as *mut Self;

        let gui = self.gui.as_mut().unwrap();

        if let Err(err) = gui.make_current() {
            log::error!("could not make gui window current: {err}");
            report_error(err);
            return;
        }
        gui.run(|ctx| (unsafe { &mut *self_ptr }).gui(ctx));
        gui.clear();
        gui.paint();

        if let Err(err) = gui.swap_buffers() {
            log::error!("could not swap gui window buffers: {err}");
            report_error(err);
            return;
        }

        let overlay = self.overlay.as_mut().unwrap();

        overlay.window().set_cursor_hittest(false).unwrap();
        if let Err(err) = overlay.make_current() {
            log::error!("could not make overlay window current: {err}");
            report_error(err);
            return;
        }

        overlay.run(move |egui_ctx| {
            (unsafe { &mut *self_ptr }).overlay(egui_ctx);
        });
        overlay.clear();
        overlay.paint();

        if let Err(err) = overlay.swap_buffers() {
            log::error!("could not swap overlay window buffers: {err}");
            report_error(err);
        }
    }
}

fn collapsing_open(ui: &mut Ui, title: &str, add_body: impl FnOnce(&mut Ui)) {
    CollapsingHeader::new(title)
        .default_open(true)
        .show(ui, add_body);
}
