use eframe::egui::{self, vec2, Align, Align2, Color32, DragValue, FontId, RichText, Sense, Ui};
use std::sync::mpsc;
use strum::IntoEnumIterator;

use crate::{
    color::{Color, Colors},
    config::{parse_config, write_config, AimbotStatus, Config, VERSION},
    constants::Constants,
    key_codes::KeyCode,
    message::Message,
    mouse::MouseStatus,
};

#[derive(PartialEq)]
pub enum Tab {
    Aimbot,
    Triggerbot,
    Unsafe,
    Colors,
}

pub struct Gui {
    tx: mpsc::Sender<Message>,
    rx: mpsc::Receiver<Message>,
    current_tab: Tab,
    config: Config,
    status: AimbotStatus,
    mouse_status: MouseStatus,
    frame_times: Vec<f64>,
    average_frame_time: f64,
}

impl Gui {
    pub fn new(tx: mpsc::Sender<Message>, rx: mpsc::Receiver<Message>) -> Self {
        // read config
        let config = parse_config();
        // override config if invalid
        write_config(&config);
        let status = AimbotStatus::GameNotStarted;
        let out = Self {
            tx,
            rx,
            current_tab: Tab::Aimbot,
            config,
            status,
            mouse_status: MouseStatus::NoMouseFound,
            frame_times: Vec::with_capacity(50),
            average_frame_time: 0.0,
        };
        write_config(&out.config);
        out
    }

    fn send_config(&self) {
        self.send_message(Message::Config(self.config.clone()));
        write_config(&self.config);
    }

    fn send_message(&self, message: Message) {
        self.tx.send(message).expect("aimbot thread died");
    }

    fn aimbot_grid(&mut self, ui: &mut Ui) {
        egui::Grid::new("aimbot").num_columns(4).show(ui, |ui| {
            ui.label("Enable Aimbot");
            if ui.checkbox(&mut self.config.aimbot.enabled, "").changed() {
                self.send_config();
            }

            ui.label("Hotkey");
            egui::ComboBox::new("aimbot_hotkey", "")
                .selected_text(format!("{:?}", self.config.aimbot.hotkey))
                .show_ui(ui, |ui| {
                    for key_code in KeyCode::iter() {
                        let text = format!("{:?}", &key_code);
                        if ui
                            .selectable_value(&mut self.config.aimbot.hotkey, key_code, text)
                            .clicked()
                        {
                            self.send_config();
                        }
                    }
                });
            ui.end_row();

            ui.label("Aim Lock");
            if ui.checkbox(&mut self.config.aimbot.aim_lock, "").changed() {
                self.send_config();
            }

            ui.label("Start Bullet");
            if ui
                .add(
                    DragValue::new(&mut self.config.aimbot.start_bullet)
                        .range(0..=10)
                        .speed(0.05),
                )
                .changed()
            {
                self.send_config();
            }
            ui.end_row();

            ui.label("Visibility Check");
            if ui
                .checkbox(&mut self.config.aimbot.visibility_check, "")
                .changed()
            {
                self.send_config();
            }

            ui.label("FOV");
            if ui
                .add(
                    DragValue::new(&mut self.config.aimbot.fov)
                        .range(0.1..=360.0)
                        .suffix("°")
                        .speed(0.02)
                        .max_decimals(1),
                )
                .changed()
            {
                self.send_config();
            }
            ui.end_row();

            ui.label("Multibone");
            if ui.checkbox(&mut self.config.aimbot.multibone, "").changed() {
                self.send_config();
            }

            ui.label("Smooth");
            if ui
                .add(
                    DragValue::new(&mut self.config.aimbot.smooth)
                        .range(1.0..=10.0)
                        .speed(0.02)
                        .max_decimals(1),
                )
                .changed()
            {
                self.send_config();
            }
            ui.end_row();

            ui.label("Flash Check");
            if ui
                .checkbox(&mut self.config.aimbot.flash_check, "")
                .changed()
            {
                self.send_config();
            }

            ui.label("Enable RCS");
            if ui.checkbox(&mut self.config.aimbot.rcs, "").changed() {
                self.send_config();
            }
            ui.end_row();
        });
    }

    fn triggerbot_grid(&mut self, ui: &mut Ui) {
        egui::Grid::new("triggerbot").num_columns(4).show(ui, |ui| {
            ui.label("Enable");
            if ui
                .checkbox(&mut self.config.triggerbot.enabled, "")
                .changed()
            {
                self.send_config();
            }

            ui.label("Hotkey");
            egui::ComboBox::new("triggerbot_hotkey", "")
                .selected_text(format!("{:?}", self.config.triggerbot.hotkey))
                .show_ui(ui, |ui| {
                    for key_code in KeyCode::iter() {
                        let text = format!("{:?}", &key_code);
                        if ui
                            .selectable_value(&mut self.config.triggerbot.hotkey, key_code, text)
                            .clicked()
                        {
                            self.send_config();
                        }
                    }
                });
            ui.end_row();

            ui.label("Min Delay");
            let end = self.config.triggerbot.delay_range.end;
            if ui
                .add(
                    DragValue::new(&mut self.config.triggerbot.delay_range.start)
                        .range(0..=end)
                        .speed(0.2),
                )
                .changed()
            {
                self.send_config();
            }

            ui.label("Max Delay");
            let start = self.config.triggerbot.delay_range.start;
            if ui
                .add(
                    DragValue::new(&mut self.config.triggerbot.delay_range.end)
                        .range(start..=1000)
                        .speed(0.2),
                )
                .changed()
            {
                self.send_config();
            }
            ui.end_row();

            ui.label("Visibility Check");
            if ui
                .checkbox(&mut self.config.triggerbot.visibility_check, "")
                .changed()
            {
                self.send_config();
            }

            ui.label("Flash Check");
            if ui
                .checkbox(&mut self.config.triggerbot.flash_check, "")
                .changed()
            {
                self.send_config();
            }
            ui.end_row();
        });
    }

    fn unsafe_grid(&mut self, ui: &mut Ui) {
        egui::Grid::new("unsafe").num_columns(4).show(ui, |ui| {
            ui.label("Glow");
            if ui.checkbox(&mut self.config.misc.glow, "").changed() {
                self.send_config();
            }

            ui.label("Friendly Glow");
            if ui
                .checkbox(&mut self.config.misc.friendly_glow, "")
                .changed()
            {
                self.send_config();
            }
            ui.end_row();

            ui.label("No Flash");
            if ui.checkbox(&mut self.config.misc.no_flash, "").changed() {
                self.send_config();
            }

            ui.label("Max Flash Alpha");
            if ui
                .add(
                    DragValue::new(&mut self.config.misc.max_flash_alpha)
                        .range(0.0..=1.0)
                        .speed(0.002)
                        .max_decimals(2),
                )
                .changed()
            {
                self.send_config();
            }
            ui.end_row();

            ui.label("FOV Changer");
            if ui.checkbox(&mut self.config.misc.fov_changer, "").changed() {
                self.send_config();
            }

            ui.label("Desired FOV");
            if ui
                .add(DragValue::new(&mut self.config.misc.desired_fov).speed(0.1))
                .changed()
            {
                self.send_config();
            }

            if ui
                .button(RichText::new("").font(FontId::monospace(12.0)))
                .clicked()
            {
                self.config.misc.desired_fov = Constants::DEFAULT_FOV;
                self.send_config();
            }
        });
    }

    fn colors_grid(&mut self, ui: &mut Ui) {
        egui::Grid::new("unsafe").num_columns(4).show(ui, |ui| {
            ui.label("Glow Enemy Color");
            if let Some(color) = self.color_picker(ui, &self.config.misc.enemy_color) {
                self.config.misc.enemy_color = color;
                self.send_config();
            }
            ui.end_row();

            ui.label("Glow Friendly Color");
            if let Some(color) = self.color_picker(ui, &self.config.misc.friendly_color) {
                self.config.misc.friendly_color = color;
                self.send_config();
            }
            ui.end_row();
        });
    }

    fn add_game_status(&mut self, ui: &mut Ui) {
        ui.horizontal_top(|ui| {
            ui.label(
                egui::RichText::new(self.status.string())
                    .line_height(Some(8.0))
                    .color(match self.status {
                        AimbotStatus::Working => Colors::GREEN,
                        AimbotStatus::GameNotStarted => Colors::YELLOW,
                    }),
            );

            let mouse_text = match &self.mouse_status {
                MouseStatus::Working(name) => name,
                MouseStatus::PermissionsRequired => {
                    "mouse input only works when user is in input group"
                }
                MouseStatus::Disconnected => "mouse was disconnected",
                MouseStatus::NoMouseFound => "no mouse was found",
            };
            let color = if let MouseStatus::Working(_) = &self.mouse_status {
                Colors::SUBTEXT
            } else {
                Colors::YELLOW
            };
            ui.label(
                egui::RichText::new(mouse_text)
                    .line_height(Some(8.0))
                    .color(color),
            );
        });
    }

    fn color_picker(&self, ui: &mut Ui, color: &Color) -> Option<Color> {
        let [mut r, mut g, mut b, mut a] = color.egui_color().to_array();
        let mut changed = false;
        if ui.add(DragValue::new(&mut r).prefix("r: ")).changed() {
            changed = true;
        }
        if ui.add(DragValue::new(&mut g).prefix("g: ")).changed() {
            changed = true;
        }
        if ui.add(DragValue::new(&mut b).prefix("b: ")).changed() {
            changed = true;
        };
        if ui.add(DragValue::new(&mut a).prefix("a: ")).changed() {
            changed = true;
        };
        let (response, painter) = ui.allocate_painter(ui.spacing().interact_size, Sense::hover());
        painter.rect_filled(
            response.rect,
            ui.style().visuals.widgets.inactive.rounding,
            color.egui_color(),
        );
        if changed {
            return Some(Color::rgba(r, g, b, a));
        }
        None
    }
}

impl eframe::App for Gui {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        // makes it more inefficient to force draw 60fps, but else the mouse disconnect message does not show up
        // todo: when update is split into tick and show, put message parsing into tick and force update the ui when message are received
        ctx.request_repaint();

        while let Ok(message) = self.rx.try_recv() {
            match message {
                Message::Status(status) => self.status = status,
                Message::MouseStatus(status) => self.mouse_status = status,
                Message::FrameTime(time) => {
                    self.frame_times.push(time);
                    if self.frame_times.len() >= self.frame_times.capacity() {
                        self.average_frame_time = self.frame_times.iter().sum::<f64>()
                            / self.frame_times.capacity() as f64;
                        self.frame_times.clear();
                    }
                }
                _ => {}
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(6.0, 4.0);
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.current_tab, Tab::Aimbot, "Aimbot");
                ui.selectable_value(&mut self.current_tab, Tab::Triggerbot, "Triggerbot");
                ui.selectable_value(&mut self.current_tab, Tab::Unsafe, "Unsafe");
                ui.selectable_value(&mut self.current_tab, Tab::Colors, "Colors");

                ui.with_layout(egui::Layout::right_to_left(Align::Min), |ui| {
                    if ui.button("Report Issues").clicked() {
                        ctx.open_url(egui::OpenUrl {
                            url: String::from("https://github.com/avitran0/deadlocked/issues"),
                            new_tab: false,
                        });
                    }
                });
            });
            ui.separator();

            self.add_game_status(ui);
            ui.separator();

            match self.current_tab {
                Tab::Aimbot => self.aimbot_grid(ui),
                Tab::Triggerbot => self.triggerbot_grid(ui),
                Tab::Unsafe => self.unsafe_grid(ui),
                Tab::Colors => self.colors_grid(ui),
            }
        });

        let font = egui::FontId::proportional(12.0);
        let text_size = ctx.fonts(|fonts| {
            fonts
                .layout_no_wrap(String::from(VERSION), font.clone(), Color32::WHITE)
                .size()
        });

        ctx.layer_painter(egui::LayerId::background()).text(
            Align2::RIGHT_BOTTOM
                .align_size_within_rect(text_size, ctx.screen_rect().shrink(4.0))
                .max,
            Align2::RIGHT_BOTTOM,
            VERSION,
            font.clone(),
            Colors::SUBTEXT,
        );

        let frame_time = format!("{} us", self.average_frame_time);
        ctx.layer_painter(egui::LayerId::background()).text(
            ctx.screen_rect().left_bottom() + vec2(4.0, -4.0),
            Align2::LEFT_BOTTOM,
            frame_time,
            font,
            Colors::SUBTEXT,
        );
    }
}
