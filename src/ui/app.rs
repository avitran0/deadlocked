use std::{
    collections::HashMap,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

use utils::{channel::Channel, sync::Mutex};
use winit::{
    application::ApplicationHandler,
    event::{ElementState, StartCause, WindowEvent},
    keyboard::NamedKey,
};

use crate::{
    config::{
        CONFIG_PATH, Config, DEFAULT_CONFIG_NAME, available_configs, parse_config, write_config,
    },
    cs2::entity::weapon::Weapon,
    data::{Data, SoundType},
    message::{GameMessage, GameStatus, UiMessage},
    ui::{
        grenades::{Grenade, GrenadeList, read_grenades},
        gui::{Tab, aimbot::AimbotTab},
        trail::Trail,
        window_context::WindowContext,
    },
};

pub struct App {
    pub gui: Option<WindowContext>,
    pub overlay: Option<WindowContext>,
    next_frame_time: Instant,
    pub show_about: bool,

    pub channel: Channel<GameMessage, UiMessage>,
    pub data: Arc<Mutex<Data>>,

    pub game_status: GameStatus,
    pub display_scale: f32,
    pub overlay_window_pos: Option<(i32, i32)>,
    pub overlay_window_size: Option<(u32, u32)>,
    pub trails: HashMap<u64, Trail>,
    pub player_sounds: HashMap<u64, (Instant, SoundType)>,

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
}

impl App {
    pub fn new(channel: Channel<GameMessage, UiMessage>, data: Arc<Mutex<Data>>) -> Self {
        // read config
        let config = parse_config(&CONFIG_PATH.join(DEFAULT_CONFIG_NAME));
        // override config if invalid
        write_config(&config, &CONFIG_PATH.join(DEFAULT_CONFIG_NAME));
        let grenades = read_grenades();

        let ret = Self {
            gui: None,
            overlay: None,

            next_frame_time: Instant::now() + frame_duration(&config),
            show_about: false,

            channel,
            data,
            config,
            current_config: CONFIG_PATH.join(DEFAULT_CONFIG_NAME),
            available_configs: available_configs(),
            new_config_name: String::new(),

            game_status: GameStatus::NotStarted,
            display_scale: 1.0,
            overlay_window_pos: None,
            overlay_window_size: None,
            trails: HashMap::new(),
            player_sounds: HashMap::new(),

            grenades,
            new_grenade: Grenade::new(),
            current_grenade: None,

            current_tab: Tab::Aimbot,
            aimbot_tab: AimbotTab::Global,
            aimbot_weapon: Weapon::Ak47,
        };
        ret.send_config();
        ret
    }

    fn create_window(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let gui = WindowContext::new(event_loop, false, self.config.accent_color);
        let overlay = WindowContext::new(event_loop, true, self.config.accent_color);

        self.display_scale = gui.window().scale_factor() as f32;
        utils::info!("detected display scale: {}", self.display_scale);

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
            self.next_frame_time += frame_duration(&self.config);
        }
    }

    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.create_window(event_loop);

        self.next_frame_time = Instant::now() + frame_duration(&self.config);
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
        while let Ok(message) = self.channel.try_receive() {
            self.game_status = message.0;
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
        let is_gui_window = gui.window().id() == window_id;

        match &window_event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(new_size) => {
                window.resize(*new_size);
            }
            WindowEvent::RedrawRequested => {
                event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(
                    self.next_frame_time,
                ));
                // Render once per frame from the GUI event to avoid duplicate work.
                if is_gui_window {
                    self.render();
                }
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
                let event_response = self.gui.as_mut().unwrap().process_event(&window_event);

                if event_response.repaint {
                    self.gui.as_ref().unwrap().request_redraw();
                    self.overlay.as_ref().unwrap().request_redraw();
                }
            }
            _ => {
                let event_response = self.gui.as_mut().unwrap().process_event(&window_event);

                if event_response.repaint {
                    self.gui.as_ref().unwrap().request_redraw();
                    self.overlay.as_ref().unwrap().request_redraw();
                }
            }
        }
    }
}

fn frame_duration(config: &Config) -> Duration {
    let hz = config.hud.overlay_refresh_rate.clamp(30, 360);
    Duration::from_micros(1_000_000 / hz)
}
