use std::{
    collections::{HashMap, VecDeque},
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
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
    audio::{Audio, builtin_sound},
    config::{
        CONFIG_PATH, Config, DEFAULT_CONFIG_NAME,
        application::{ApplicationConfig, read_app_config, write_app_config},
        available_configs,
        hud::HitmarkerSound,
        parse_config, write_config,
    },
    message::{GameMessage, GameStatus, RadarMessage, RadarStatus, UiMessage},
    os::is_omarchy,
    parser::bvh::Bvh,
    ui::{
        grenades::{Grenade, GrenadeList, read_grenades},
        gui::{Tab, aimbot::AimbotTab},
        overlay::model::ModelRenderer,
        trail::Trail,
        window_context::WindowContext,
    },
    update,
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
    pub audio: Audio,
    pub hit_custom_sound: Option<Vec<u8>>,
    pub hit_custom_sound_warning: Option<String>,
    pub kill_custom_sound: Option<Vec<u8>>,
    pub kill_custom_sound_warning: Option<String>,
    pub last_hit_sequence: u64,
    pub last_kill_sequence: u64,
    pub hitmarker_started: Option<Instant>,
    pub hitmarker_kill: bool,

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
    pub debug_popup: bool,
    pub update_popup: bool,
    pub omarchy_popup: bool,
    pub overlay_egui: Option<egui::Context>,
    pub model_renderer: Option<Arc<ModelRenderer>>,
    pub bvh: Option<Bvh>,
    pub bvh_map: String,
    pub bvh_build_date: String,
    pub last_bvh_load: Option<Instant>,

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
        let mut app_config = read_app_config();
        let config_name = Path::new(&app_config.config_name)
            .file_name()
            .filter(|name| name.to_string_lossy() == app_config.config_name)
            .filter(|name| name.to_string_lossy().ends_with(".toml"))
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| DEFAULT_CONFIG_NAME.to_owned());
        if app_config.config_name != config_name {
            app_config.config_name = config_name.clone();
            write_app_config(&app_config);
        }

        let current_config = CONFIG_PATH.join(&config_name);
        let config = parse_config(&current_config);
        write_config(&config, &current_config);
        let grenades = read_grenades();
        write_app_config(&app_config);

        let update_status = update::check();
        let update_popup = matches!(update_status, UpdateStatus::Available { .. });

        let audio = Audio::new();
        let mut state = Self {
            channel_game,
            channel_radar,
            data,
            app_config,
            config,
            current_config,
            available_configs: available_configs(),
            new_config_name: String::new(),
            game_status: GameStatus::NotStarted,
            display_scale: 1.0,
            trails: HashMap::new(),
            player_sounds: HashMap::new(),
            frame_times: VecDeque::with_capacity(500),
            audio,
            hit_custom_sound: None,
            hit_custom_sound_warning: None,
            kill_custom_sound: None,
            kill_custom_sound_warning: None,
            last_hit_sequence: 0,
            last_kill_sequence: 0,
            hitmarker_started: None,
            hitmarker_kill: false,
            grenades,
            new_grenade: Grenade::new(),
            current_grenade: None,
            current_tab: Tab::Aimbot,
            aimbot_tab: AimbotTab::Global,
            aimbot_weapon: Weapon::AK47,
            update_status,
            text_popup: None,
            debug_popup: cfg!(debug_assertions),
            update_popup,
            omarchy_popup: is_omarchy(),
            overlay_egui: None,
            model_renderer: None,
            bvh: None,
            bvh_map: String::new(),
            bvh_build_date: String::new(),
            last_bvh_load: None,
            radar_status: RadarStatus::Disabled,
        };
        state.reload_custom_sound();
        state
    }

    pub fn reload_custom_sound(&mut self) {
        if self.config.hud.hitmarker.hit_sound != HitmarkerSound::Custom {
            self.hit_custom_sound = None;
            self.hit_custom_sound_warning = None;
        } else {
            match std::fs::read(&self.config.hud.hitmarker.hit_custom_sound_path) {
                Ok(bytes) => match Audio::validate(bytes.clone()) {
                    Ok(()) => {
                        self.hit_custom_sound = Some(bytes);
                        self.hit_custom_sound_warning = None;
                    }
                    Err(error) => {
                        self.hit_custom_sound = None;
                        self.hit_custom_sound_warning =
                            Some(format!("Custom sound could not be decoded: {error}"));
                    }
                },
                Err(error) => {
                    self.hit_custom_sound = None;
                    self.hit_custom_sound_warning =
                        Some(format!("Custom sound could not be loaded: {error}"));
                }
            }
        }

        if self.config.hud.hitmarker.kill_sound != HitmarkerSound::Custom {
            self.kill_custom_sound = None;
            self.kill_custom_sound_warning = None;
        } else {
            match std::fs::read(&self.config.hud.hitmarker.kill_custom_sound_path) {
                Ok(bytes) => match Audio::validate(bytes.clone()) {
                    Ok(()) => {
                        self.kill_custom_sound = Some(bytes);
                        self.kill_custom_sound_warning = None;
                    }
                    Err(error) => {
                        self.kill_custom_sound = None;
                        self.kill_custom_sound_warning =
                            Some(format!("Custom sound could not be decoded: {error}"));
                    }
                },
                Err(error) => {
                    self.kill_custom_sound = None;
                    self.kill_custom_sound_warning =
                        Some(format!("Custom sound could not be loaded: {error}"));
                }
            }
        }
    }

    pub fn play_hitmarker_test(&self, kill: bool) {
        let sound = (if kill {
            &self.kill_custom_sound
        } else {
            &self.hit_custom_sound
        })
        .as_deref()
        .unwrap_or_else(|| {
            builtin_sound(if kill {
                self.config.hud.hitmarker.kill_sound
            } else {
                self.config.hud.hitmarker.hit_sound
            })
        });
        self.audio.play(
            sound,
            if kill {
                self.config.hud.hitmarker.kill_volume
            } else {
                self.config.hud.hitmarker.hit_volume
            },
        );
    }

    pub fn update_hitmarker(&mut self, in_game: bool, hit_sequence: u64, kill_sequence: u64) {
        if !in_game {
            self.last_hit_sequence = hit_sequence;
            self.last_kill_sequence = kill_sequence;
            self.hitmarker_started = None;
            return;
        }

        let hit_count = hit_sequence.saturating_sub(self.last_hit_sequence);
        let kill_count = kill_sequence.saturating_sub(self.last_kill_sequence);
        self.last_hit_sequence = hit_sequence;
        self.last_kill_sequence = kill_sequence;
        if hit_count == 0 && kill_count == 0 {
            return;
        }
        self.hitmarker_started = Some(Instant::now());
        self.hitmarker_kill = kill_count > 0;

        if self.config.hud.hitmarker.hit_sound_enabled {
            if !self.hitmarker_kill {
                let sound = self
                    .hit_custom_sound
                    .as_deref()
                    .unwrap_or_else(|| builtin_sound(self.config.hud.hitmarker.hit_sound));
                const MAX_SOUNDS_PER_FRAME: u64 = 4;
                let sound_count = hit_count.max(kill_count).min(MAX_SOUNDS_PER_FRAME);
                for _ in 0..sound_count {
                    self.audio.play(sound, self.config.hud.hitmarker.hit_volume);
                }
            }
        }
        if self.config.hud.hitmarker.kill_sound_enabled {
            let sound = self
                .kill_custom_sound
                .as_deref()
                .unwrap_or_else(|| builtin_sound(self.config.hud.hitmarker.kill_sound));
            const MAX_SOUNDS_PER_FRAME: u64 = 4;
            let sound_count = hit_count.max(kill_count).min(MAX_SOUNDS_PER_FRAME);
            for _ in 0..sound_count {
                self.audio
                    .play(sound, self.config.hud.hitmarker.kill_volume);
            }
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
        ret.send_grenades_game();
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

    fn render_gui(&mut self) {
        let state = &mut self.state;
        let gui = self.gui.as_mut().unwrap();

        gui.make_current().unwrap();
        gui.run(|ui| state.gui(ui));
        gui.clear();
        gui.paint();
        gui.swap_buffers().unwrap();

        if gui.egui().has_requested_repaint() {
            gui.window().request_redraw();
        }
    }

    fn render_overlay(&mut self) {
        let state = &mut self.state;
        let overlay = self.overlay.as_mut().unwrap();

        overlay.window().set_cursor_hittest(false).unwrap();
        {
            let data = state.data.lock();
            Self::update_overlay_window(overlay, &data);
        }
        overlay.make_current().unwrap();
        let glow = overlay.glow();
        overlay.run(|ui| state.overlay(ui, &glow));
        overlay.clear();
        overlay.paint();
        overlay.swap_buffers().unwrap();
    }

    fn receive_events(&mut self) {
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
            self.state.radar_status = message;
        }
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

            self.receive_events();
            self.render_overlay();

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
        self.gui.as_ref().unwrap().window().request_redraw();
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        self.receive_events();

        let gui_window_id = self.gui.as_ref().map(|gui| gui.window().id());
        let overlay_window_id = self.overlay.as_ref().map(|overlay| overlay.window().id());
        let is_gui = gui_window_id == Some(window_id);
        let is_overlay = overlay_window_id == Some(window_id);

        if !is_gui && !is_overlay {
            return;
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(new_size) => {
                if is_gui {
                    let Some(gui) = self.gui.as_mut() else { return };
                    gui.resize(new_size);
                    let response = gui.process_event(&WindowEvent::Resized(new_size));
                    if response.repaint {
                        gui.window().request_redraw();
                    }
                } else if is_overlay {
                    let Some(overlay) = self.overlay.as_mut() else {
                        return;
                    };
                    overlay.resize(new_size);
                    let response = overlay.process_event(&WindowEvent::Resized(new_size));
                    if response.repaint {
                        overlay.window().request_redraw();
                    }
                }
            }
            WindowEvent::RedrawRequested if is_gui => self.render_gui(),
            _ if is_gui => {
                let Some(gui) = self.gui.as_mut() else { return };
                if let WindowEvent::KeyboardInput {
                    event,
                    is_synthetic: false,
                    ..
                } = &event
                    && let winit::keyboard::Key::Named(key) = event.logical_key
                {
                    let modifiers = match key {
                        NamedKey::Control => Some(egui::Modifiers::CTRL),
                        NamedKey::Shift => Some(egui::Modifiers::SHIFT),
                        NamedKey::Alt => Some(egui::Modifiers::ALT),
                        _ => None,
                    };
                    if let Some(modifiers) = modifiers {
                        gui.process_modifier(
                            modifiers,
                            event.state == ElementState::Pressed,
                            event.repeat,
                        );
                    }
                }
                let response = gui.process_event(&event);
                if response.repaint {
                    gui.window().request_redraw();
                }
            }
            _ => {}
        }
    }

    fn exiting(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        let (Some(overlay), Some(renderer)) =
            (self.overlay.as_ref(), self.state.model_renderer.as_ref())
        else {
            return;
        };

        if let Err(error) = overlay.make_current() {
            utils::warn!("could not make overlay context current for model cleanup: {error}");
            return;
        }
        renderer.destroy();
    }
}
