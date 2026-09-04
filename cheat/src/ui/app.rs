use std::{
    collections::{HashMap, VecDeque},
    ops::{Deref, DerefMut},
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

use shared::{Data, SoundType, Weapon};
use utils::{Channel, Mutex};
use winit::{
    application::ApplicationHandler,
    event::{ElementState, StartCause, WindowEvent},
    keyboard::NamedKey,
};

use crate::{
    config::{
        CONFIG_PATH, Config, DEFAULT_CONFIG_NAME,
        application::{ApplicationConfig, read_app_config},
        available_configs, parse_config, write_config,
    },
    message::{GameMessage, GameStatus, RadarMessage, RadarStatus, UiMessage},
    ui::{
        grenades::{Grenade, GrenadeList, read_grenades},
        gui::{Tab, aimbot::AimbotTab},
        trail::Trail,
        window_context::WindowContext,
    },
    update::UpdateStatus,
};

pub struct AppState {
    pub channel_game: Channel<GameMessage, UiMessage>,
    pub channel_radar: Channel<RadarMessage, RadarStatus>,
    pub data: Arc<Mutex<Data>>,

    pub game_status: GameStatus,
    pub display_scale: f32,
    pub trails: HashMap<usize, Trail>,
    pub player_sounds: HashMap<u64, (Instant, SoundType)>,
    pub frame_times: VecDeque<Duration>,

    pub grenades: GrenadeList,
    pub new_grenade: Grenade,
    pub current_grenade: Option<(String, usize)>,

    #[allow(dead_code)]
    pub app_config: ApplicationConfig,
    pub config: Config,
    pub current_config: PathBuf,
    pub available_configs: Vec<PathBuf>,
    pub new_config_name: String,

    pub current_tab: Tab,
    pub aimbot_tab: AimbotTab,
    pub aimbot_weapon: Weapon,

    pub update_status: UpdateStatus,

    pub text_popup: Option<String>,
    pub update_popup: bool,
    pub overlay_egui: Option<egui::Context>,

    pub radar_status: RadarStatus,
}

pub struct App {
    pub gui: Option<WindowContext>,
    pub overlay: Option<WindowContext>,
    next_frame_time: Instant,
    pub state: AppState,
}

impl Deref for App {
    type Target = AppState;
    fn deref(&self) -> &AppState {
        &self.state
    }
}

impl DerefMut for App {
    fn deref_mut(&mut self) -> &mut AppState {
        &mut self.state
    }
}

impl AppState {
    pub fn new(
        channel_game: Channel<GameMessage, UiMessage>,
        channel_radar: Channel<RadarMessage, RadarStatus>,
        data: Arc<Mutex<Data>>,
    ) -> Self {
        let config = parse_config(&CONFIG_PATH.join(DEFAULT_CONFIG_NAME));
        write_config(&config, &CONFIG_PATH.join(DEFAULT_CONFIG_NAME));
        let grenades = read_grenades();
        let app_config = read_app_config();

        let update_status = crate::update::check();
        let update_popup = matches!(update_status, crate::update::UpdateStatus::Available { .. });

        Self {
            channel_game,
            channel_radar,
            data,
            app_config,
            config,
            current_config: CONFIG_PATH.join(DEFAULT_CONFIG_NAME),
            available_configs: available_configs(),
            new_config_name: String::new(),
            game_status: GameStatus::NotStarted,
            display_scale: 1.0,
            trails: HashMap::new(),
            player_sounds: HashMap::new(),
            frame_times: VecDeque::with_capacity(500),
            grenades,
            new_grenade: Grenade::new(),
            current_grenade: None,
            current_tab: Tab::Aimbot,
            aimbot_tab: AimbotTab::Global,
            aimbot_weapon: Weapon::AK47,
            update_status,
            text_popup: None,
            update_popup,
            overlay_egui: None,
            radar_status: RadarStatus::Disabled,
        }
    }
}

impl App {
    pub fn new(
        channel_game: Channel<GameMessage, UiMessage>,
        channel_radar: Channel<RadarMessage, RadarStatus>,
        data: Arc<Mutex<Data>>,
    ) -> Self {
        let state = AppState::new(channel_game, channel_radar, data);
        let ret = Self {
            gui: None,
            overlay: None,
            next_frame_time: Instant::now() + Duration::from_millis(16),
            state,
        };
        ret.send_config_game();
        ret.send_config_radar();
        ret
    }

    pub fn create_window(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let gui = WindowContext::new(event_loop, false, self.state.config.accent_color);
        let overlay = WindowContext::new(event_loop, true, self.state.config.accent_color);

        self.state.config.font.set(gui.egui());
        self.state.config.font.set(overlay.egui());

        self.state.display_scale = gui.window().scale_factor() as f32;
        self.state.overlay_egui = Some(overlay.egui().clone());
        utils::info!("detected display scale: {}", self.state.display_scale);

        self.gui = Some(gui);
        self.overlay = Some(overlay);
    }

    fn frame_duration(&self) -> Duration {
        Duration::from_secs_f32(1.0 / self.state.config.fps as f32)
    }
}

impl ApplicationHandler for App {
    fn new_events(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, cause: StartCause) {
        if let StartCause::ResumeTimeReached { .. } = cause {
            self.next_frame_time += self.frame_duration();

            let now = Instant::now();
            if self.next_frame_time < now {
                self.next_frame_time = now + self.frame_duration();
            }

            self.render();

            event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(
                self.next_frame_time,
            ));
        }
    }

    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.create_window(event_loop);

        self.next_frame_time = Instant::now() + self.frame_duration();
        event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(
            self.next_frame_time,
        ));
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        window_event: WindowEvent,
    ) {
        while let Ok(message) = self.state.channel_game.try_receive() {
            match message {
                UiMessage::Status(status) => self.state.game_status = status,
                UiMessage::FrameTime(time) => {
                    if self.state.frame_times.len() >= 500 {
                        self.state.frame_times.pop_front();
                    }
                    self.state.frame_times.push_back(time);
                }
            }
        }

        while let Ok(message) = self.state.channel_radar.try_receive() {
            self.radar_status = message;
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

        match &window_event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(new_size) => {
                window.resize(*new_size);
            }
            WindowEvent::RedrawRequested => {
                if !self
                    .gui
                    .as_ref()
                    .map(|window| window.window().id() == window_id)
                    .unwrap_or_default()
                {
                    return;
                }
                self.render();
            }
            WindowEvent::KeyboardInput {
                event,
                is_synthetic: false,
                ..
            } => {
                if let winit::keyboard::Key::Named(key) = event.logical_key {
                    let modifiers = match key {
                        NamedKey::Control => Some(egui::Modifiers::CTRL),
                        NamedKey::Shift => Some(egui::Modifiers::SHIFT),
                        NamedKey::Alt => Some(egui::Modifiers::ALT),
                        _ => None,
                    };

                    if let Some(modifiers) = modifiers {
                        self.gui.as_mut().unwrap().process_modifier(
                            modifiers,
                            event.state == ElementState::Pressed,
                            event.repeat,
                        );
                    }
                }
                let _ = self
                    .gui
                    .as_mut()
                    .map(|gui| gui.process_event(&window_event));
            }
            _ => {
                let _ = self
                    .gui
                    .as_mut()
                    .map(|gui| gui.process_event(&window_event));
            }
        }
    }
}
