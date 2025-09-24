use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use arboard::Clipboard;
use crossbeam::channel::{Receiver, Sender};
use egui::{FontData, FontDefinitions, Stroke, Style};
use egui_glow::glow;
use winit::{
    application::ApplicationHandler,
    event::{StartCause, WindowEvent},
};

use crate::{
    bvh::Bvh,
    color::Colors,
    config::{
        CONFIG_PATH, Config, DEFAULT_CONFIG_NAME, available_configs, parse_config, write_config,
    },
    cs2::weapon::Weapon,
    data::Data,
    gui::{AimbotTab, Tab},
    message::{Envelope, GameStatus, Message, RadarStatus, Target},
    mouse::DeviceStatus,
    window_context::WindowContext,
};

const FRAME_RATE: u64 = 120;
const FRAME_DURATION: Duration = Duration::from_micros(1_000_000 / FRAME_RATE);

pub struct App {
    pub gui_window: Option<WindowContext>,
    pub gui_gl: Option<Arc<glow::Context>>,
    pub gui_glow: Option<egui_glow::EguiGlow>,
    pub overlay_window: Option<WindowContext>,
    pub overlay_gl: Option<Arc<glow::Context>>,
    pub overlay_glow: Option<egui_glow::EguiGlow>,
    pub clipboard: Clipboard,
    next_frame_time: Instant,

    pub tx: Sender<Envelope>,
    pub rx: Receiver<Message>,
    pub data: Arc<Mutex<Data>>,
    #[allow(unused)]
    pub bvh: Arc<Mutex<HashMap<String, Bvh>>>,

    pub game_status: GameStatus,
    pub mouse_status: DeviceStatus,
    pub selected_mouse: Option<String>,
    pub radar_status: RadarStatus,
    pub display_scale: f32,

    pub config: Config,
    pub current_config: PathBuf,
    pub available_configs: Vec<PathBuf>,
    pub new_config_name: String,

    pub current_tab: Tab,
    pub aimbot_tab: AimbotTab,
    pub aimbot_weapon: Weapon,
}

impl App {
    pub fn new(
        tx: Sender<Envelope>,
        rx: Receiver<Message>,
        data: Arc<Mutex<Data>>,
        bvh: Arc<Mutex<HashMap<String, Bvh>>>,
    ) -> Self {
        // read config
        let config = parse_config(&CONFIG_PATH.join(DEFAULT_CONFIG_NAME));
        // override config if invalid
        write_config(&config, &CONFIG_PATH.join(DEFAULT_CONFIG_NAME));

        let ret = Self {
            gui_window: None,
            gui_gl: None,
            gui_glow: None,

            overlay_window: None,
            overlay_gl: None,
            overlay_glow: None,

            clipboard: Clipboard::new().unwrap(),
            next_frame_time: Instant::now() + FRAME_DURATION,

            tx,
            rx,
            data,
            bvh,
            config,
            current_config: CONFIG_PATH.join(DEFAULT_CONFIG_NAME),
            available_configs: available_configs(),
            new_config_name: String::new(),

            game_status: GameStatus::GameNotStarted,
            mouse_status: DeviceStatus::Disconnected,
            radar_status: RadarStatus::Disconnected,
            display_scale: 1.0,

            selected_mouse: None,

            current_tab: Tab::Aimbot,
            aimbot_tab: AimbotTab::Global,
            aimbot_weapon: Weapon::Ak47,
        };
        ret.send_config();
        ret.send_message(
            Message::RadarSetEnabled(ret.config.radar.enabled),
            Target::Radar,
        );
        ret.send_message(
            Message::ChangeRadarUrl(ret.config.radar.url.clone()),
            Target::Radar,
        );
        ret
    }

    fn create_window(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let (gui_window, gui_gl) = create_display(event_loop, false);
        let gui_gl = Arc::new(gui_gl);
        let mut gui_glow = egui_glow::EguiGlow::new(event_loop, gui_gl.clone(), None, None, true);
        prep_ctx(&mut gui_glow.egui_ctx);
        self.display_scale = gui_window.window().scale_factor() as f32;
        log::info!("detected display scale: {}", self.display_scale);

        let (overlay_window, overlay_gl) = create_display(event_loop, true);
        let overlay_gl = Arc::new(overlay_gl);
        let mut overlay_glow =
            egui_glow::EguiGlow::new(event_loop, overlay_gl.clone(), None, None, true);
        prep_ctx(&mut overlay_glow.egui_ctx);

        self.gui_window = Some(gui_window);
        self.gui_gl = Some(gui_gl);
        self.gui_glow = Some(gui_glow);

        self.overlay_window = Some(overlay_window);
        self.overlay_gl = Some(overlay_gl);
        self.overlay_glow = Some(overlay_glow);
    }
}

impl ApplicationHandler for App {
    fn new_events(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop, cause: StartCause) {
        if let StartCause::ResumeTimeReached { .. } = cause {
            if let Some(window) = &self.gui_window {
                window.window().request_redraw();
            }
            if let Some(window) = &self.overlay_window {
                window.window().request_redraw();
            }
            self.next_frame_time += FRAME_DURATION;
        }
    }

    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.create_window(event_loop);

        self.next_frame_time = Instant::now() + FRAME_DURATION;
        event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(
            self.next_frame_time,
        ));
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        while let Ok(message) = self.rx.try_recv() {
            match message {
                Message::GameStatus(status) => self.game_status = status,
                Message::MouseStatus(status) => self.mouse_status = status,
                Message::RadarStatus(status) => self.radar_status = status,
                _ => {}
            }
        }

        let Some(gui_window) = &self.gui_window else {
            return;
        };
        let Some(overlay_window) = &self.overlay_window else {
            return;
        };

        let window = if gui_window.window().id() == window_id {
            gui_window
        } else if overlay_window.window().id() == window_id {
            overlay_window
        } else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(new_size) => {
                window.resize(new_size);
            }
            WindowEvent::RedrawRequested => {
                event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(
                    self.next_frame_time,
                ));
                self.render();
            }
            _ => {
                let event_response = self
                    .gui_glow
                    .as_mut()
                    .unwrap()
                    .on_window_event(self.gui_window.as_mut().unwrap().window(), &event);

                if event_response.repaint {
                    self.gui_window.as_mut().unwrap().window().request_redraw();
                    self.overlay_window
                        .as_mut()
                        .unwrap()
                        .window()
                        .request_redraw();
                }
            }
        }
    }
}

fn create_display(
    event_loop: &winit::event_loop::ActiveEventLoop,
    overlay: bool,
) -> (WindowContext, glow::Context) {
    let glutin_window_context = WindowContext::new(event_loop, overlay);
    let gl = unsafe {
        glow::Context::from_loader_function(|s| {
            let s = std::ffi::CString::new(s)
                .expect("failed to construct C string from string for gl proc address");

            glutin_window_context.get_proc_address(&s)
        })
    };

    (glutin_window_context, gl)
}

fn prep_ctx(ctx: &mut egui::Context) {
    // add font
    let fira_sans = include_bytes!("../resources/FiraSansIcons.ttf");
    let cs2_icons = include_bytes!("../resources/CS2GunIcons.ttf");
    let mut font_definitions = FontDefinitions::default();
    font_definitions.font_data.insert(
        String::from("fira_sans"),
        Arc::new(FontData::from_static(fira_sans)),
    );
    font_definitions.font_data.insert(
        String::from("cs2_icons"),
        Arc::new(FontData::from_static(cs2_icons)),
    );

    // insert into font definitions, so it gets chosen as default
    font_definitions
        .families
        .get_mut(&egui::FontFamily::Proportional)
        .unwrap()
        .insert(0, String::from("fira_sans"));
    font_definitions
        .families
        .get_mut(&egui::FontFamily::Monospace)
        .unwrap()
        .insert(0, String::from("cs2_icons"));

    ctx.set_fonts(font_definitions);

    ctx.style_mut_of(egui::Theme::Dark, gui_style);
}

fn gui_style(style: &mut Style) {
    style.interaction.selectable_labels = false;
    for font in style.text_styles.iter_mut() {
        font.1.size = 16.0;
    }
    //style.visuals.override_text_color = Some(Color32::WHITE);

    style.visuals.window_fill = Colors::BASE;
    style.visuals.panel_fill = Colors::BASE;
    style.visuals.extreme_bg_color = Colors::BACKDROP;

    let bg_stroke = Stroke::new(1.0, Colors::SUBTEXT);
    let fg_stroke = Stroke::new(1.0, Colors::TEXT);
    let dark_stroke = Stroke::new(1.0, Colors::BASE);

    style.visuals.selection.bg_fill = Colors::BLUE;
    style.visuals.selection.stroke = dark_stroke;

    style.visuals.widgets.active.bg_fill = Colors::HIGHLIGHT;
    style.visuals.widgets.active.bg_stroke = bg_stroke;
    style.visuals.widgets.active.fg_stroke = fg_stroke;
    style.visuals.widgets.active.weak_bg_fill = Colors::HIGHLIGHT;

    style.visuals.widgets.hovered.bg_fill = Colors::HIGHLIGHT;
    style.visuals.widgets.hovered.bg_stroke = bg_stroke;
    style.visuals.widgets.hovered.fg_stroke = fg_stroke;
    style.visuals.widgets.hovered.weak_bg_fill = Colors::HIGHLIGHT;

    style.visuals.widgets.inactive.bg_fill = Colors::HIGHLIGHT;
    style.visuals.widgets.inactive.fg_stroke = fg_stroke;
    style.visuals.widgets.inactive.weak_bg_fill = Colors::HIGHLIGHT;

    style.visuals.widgets.noninteractive.bg_fill = Colors::HIGHLIGHT;
    style.visuals.widgets.noninteractive.fg_stroke = fg_stroke;
    style.visuals.widgets.noninteractive.weak_bg_fill = Colors::HIGHLIGHT;

    style.visuals.widgets.open.bg_fill = Colors::HIGHLIGHT;
    style.visuals.widgets.open.bg_stroke = bg_stroke;
    style.visuals.widgets.open.fg_stroke = fg_stroke;
    style.visuals.widgets.open.weak_bg_fill = Colors::HIGHLIGHT;
}
