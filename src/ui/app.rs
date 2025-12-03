use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use arboard::Clipboard;
use crossbeam::channel::{Receiver, Sender};
use winit::{
    application::ApplicationHandler,
    event::{StartCause, WindowEvent},
};

use crate::{
    config::{
        CONFIG_PATH, Config, DEFAULT_CONFIG_NAME, available_configs, parse_config, write_config,
    },
    cs2::entity::weapon::Weapon,
    data::Data,
    message::{Envelope, GameStatus, Message, RadarStatus, Target},
    os::mouse::DeviceStatus,
    parser::bvh::Bvh,
    ui::{
        grenades::{Grenade, GrenadeList, read_grenades},
        gui::{Tab, aimbot::AimbotTab},
        trail::Trail,
        window_context::WindowContext,
    },
};
use egui::Color32;

const FRAME_RATE: u64 = 120;
const FRAME_DURATION: Duration = Duration::from_micros(1_000_000 / FRAME_RATE);

pub struct App {
    pub gui: Option<WindowContext>,
    pub overlay: Option<WindowContext>,
    pub clipboard: Clipboard,
    next_frame_time: Instant,

    pub tx: Sender<Envelope>,
    pub rx: Receiver<Message>,
    #[allow(unused)]
    pub data: Arc<Mutex<Data>>,
    #[allow(unused)]
    pub bvh: Arc<Mutex<HashMap<String, Bvh>>>,

    pub game_status: GameStatus,
    pub mouse_status: DeviceStatus,
    pub selected_mouse: Option<String>,
    pub radar_status: RadarStatus,
    pub display_scale: f32,
    #[allow(unused)]
    pub trails: HashMap<u64, Trail>,

    pub grenades: GrenadeList,
    pub new_grenade: Grenade,
    pub current_grenade: Option<(String, usize)>,

    pub config: Config,
    pub current_config: PathBuf,
    pub available_configs: Vec<PathBuf>,
    pub new_config_name: String,

    pub current_tab: Tab,
    pub aimbot_tab: AimbotTab,
    pub aimbot_weapon: Weapon,

    // Logo animation settings (UI-state stored in the App)
    pub logo_animated: bool,
    pub logo_color_a: Color32,
    pub logo_color_b: Color32,
    pub logo_animation_start: Instant,
    pub logo_animation_speed: f32,

    // Custom logo text and manual split control
    pub logo_text: String,
    pub logo_manual_split: bool,
    pub logo_split_index: i32,
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

        let grenades = read_grenades();

        let default_logo = "DeadLocked".to_owned();
        let default_len = default_logo.chars().count();
        let default_split = ((default_len + 1) / 2) as i32;

        let ret = Self {
            gui: None,
            overlay: None,

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
            trails: HashMap::new(),

            grenades,
            new_grenade: Grenade::new(),
            current_grenade: None,

            selected_mouse: None,

            current_tab: Tab::Aimbot,
            aimbot_tab: AimbotTab::Global,
            aimbot_weapon: Weapon::Ak47,

            // default logo animation state
            logo_animated: false,
            logo_color_a: Color32::from_rgb(240, 100, 100), // red-ish
            logo_color_b: Color32::WHITE,                   // white
            logo_animation_start: Instant::now(),
            logo_animation_speed: 1.0, // cycles per second

            // default logo text + manual split defaults
            logo_text: default_logo,
            logo_manual_split: false,
            logo_split_index: default_split,
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
        let gui = WindowContext::new(event_loop, false, self.config.accent_color);
        let overlay = WindowContext::new(event_loop, true, self.config.accent_color);

        self.display_scale = gui.window().scale_factor() as f32;
        log::info!("detected display scale: {}", self.display_scale);

        self.gui = Some(gui);
        self.overlay = Some(overlay);
    }
}

impl ApplicationHandler for App {
    fn new_events(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop, cause: StartCause) {
        if let StartCause::ResumeTimeReached { .. } = cause {
            if let Some(window) = &self.gui {
                window.window().request_redraw();
            }
            if let Some(window) = &self.overlay {
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
        #[allow(unused)] window_id: winit::window::WindowId,
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

        let Some(gui) = &self.gui else {
            return;
        };
        let Some(overlay) = &self.overlay else {
            return;
        };

        let window = if gui.window().id() == window_id {
            gui
        } else if overlay.window().id() == window_id {
            overlay
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
                let event_response = self.gui.as_mut().unwrap().process_event(&event);

                if event_response.repaint {
                    self.gui.as_ref().unwrap().request_redraw();
                    self.overlay.as_ref().unwrap().request_redraw();
                }
            }
        }
    }
}
