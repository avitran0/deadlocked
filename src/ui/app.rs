use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::Arc,
    time::{Duration, Instant},
};

use arboard::Clipboard;
use crossbeam::channel::{Receiver, Sender};
use utils::{log, sync::Mutex};
use winit::{
    application::ApplicationHandler,
    event::{StartCause, WindowEvent},
};

use crate::{
    config::{
        CONFIG_PATH, Config, DEFAULT_CONFIG_NAME, available_configs, parse_config, write_config,
    },
    cs2::entity::weapon::Weapon,
    data::{Data, SoundType},
    message::{Envelope, GameStatus, Message, RadarStatus, Target},
    ui::{
        grenades::{Grenade, GrenadeList},
        gui::{Tab, aimbot::AimbotTab},
        trail::Trail,
        window_context::WindowContext,
    },
};

pub struct App {
    pub gui: Option<WindowContext>,
    pub overlay: Option<WindowContext>,
    pub clipboard: Clipboard,
    next_frame_time: Instant,
    next_kde_check: Instant,

    pub tx: Sender<Envelope>,
    pub rx: Receiver<Message>,
    pub data: Arc<Mutex<Data>>,

    pub game_status: GameStatus,
    pub radar_status: RadarStatus,
    pub display_scale: f32,
    pub overlay_window_pos: Option<(i32, i32)>,
    pub overlay_window_size: Option<(u32, u32)>,
    pub overlay_visible: Option<bool>,
    pub kde_compositing_active: bool,
    pub trails: HashMap<u64, Trail>,
    pub player_sounds: HashMap<u64, (Instant, SoundType)>,

    pub grenades: Arc<Mutex<GrenadeList>>,
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
    pub fn new(
        tx: Sender<Envelope>,
        rx: Receiver<Message>,
        data: Arc<Mutex<Data>>,
        grenades: Arc<Mutex<GrenadeList>>,
    ) -> Self {
        // read config
        let config = parse_config(&CONFIG_PATH.join(DEFAULT_CONFIG_NAME));
        // override config if invalid
        write_config(&config, &CONFIG_PATH.join(DEFAULT_CONFIG_NAME));

        let ret = Self {
            gui: None,
            overlay: None,

            clipboard: Clipboard::new().unwrap(),
            next_frame_time: Instant::now() + frame_duration(&config),
            next_kde_check: Instant::now(),

            tx,
            rx,
            data,
            config,
            current_config: CONFIG_PATH.join(DEFAULT_CONFIG_NAME),
            available_configs: available_configs(),
            new_config_name: String::new(),

            game_status: GameStatus::NotStarted,
            radar_status: RadarStatus::Disconnected,
            display_scale: 1.0,
            overlay_window_pos: None,
            overlay_window_size: None,
            overlay_visible: None,
            kde_compositing_active: true,
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
        ret.send_radar_config();
        ret
    }

    fn send_radar_config(&self) {
        self.send_message(
            Message::RadarSetEnabled(self.config.radar.enabled),
            Target::Radar,
        );
        self.send_message(
            Message::ChangeRadarUrl(self.config.radar.url.clone()),
            Target::Radar,
        );
    }

    fn create_window(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        configure_kde_compositor_policy();

        let gui = WindowContext::new(event_loop, false, self.config.accent_color);
        let overlay = WindowContext::new(event_loop, true, self.config.accent_color);

        self.display_scale = gui.window().scale_factor() as f32;
        log::info!("detected display scale: {}", self.display_scale);

        self.gui = Some(gui);
        self.overlay = Some(overlay);
        self.refresh_kde_compositor_state();
        self.set_overlay_visible(true);
    }

    pub fn overlay_allowed(&self) -> bool {
        !is_kde_session() || self.kde_compositing_active
    }

    pub fn set_overlay_visible(&mut self, visible: bool) {
        if self.overlay_visible == Some(visible) {
            return;
        }
        if let Some(overlay) = &self.overlay {
            overlay.window().set_visible(visible);
        }
        self.overlay_visible = Some(visible);
    }

    fn refresh_kde_compositor_state(&mut self) {
        if !is_kde_session() {
            self.kde_compositing_active = true;
            return;
        }

        let now = Instant::now();
        if now < self.next_kde_check {
            return;
        }
        self.next_kde_check = now + Duration::from_secs(5);

        let was_active = self.kde_compositing_active;
        let Some(active) = query_kde_compositing_active() else {
            return;
        };
        self.kde_compositing_active = active;

        if active {
            return;
        }

        if was_active {
            log::warn!("KWin compositing is disabled, re-applying overlay compositor policy");
        }
        configure_kde_compositor_policy();
        if let Some(active_after) = query_kde_compositing_active() {
            self.kde_compositing_active = active_after;
        }
    }
}

fn is_kde_session() -> bool {
    let desktop = std::env::var("XDG_CURRENT_DESKTOP")
        .unwrap_or_default()
        .to_ascii_lowercase();
    desktop.contains("kde")
        || desktop.contains("plasma")
        || std::env::var_os("KDE_FULL_SESSION").is_some()
        || std::env::var_os("KDE_SESSION_VERSION").is_some()
}

fn configure_kde_compositor_policy() {
    if !is_kde_session() {
        return;
    }

    let base_config = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")));
    let Some(config_dir) = base_config else {
        return;
    };

    let kwinrc_path = config_dir.join("kwinrc");
    if !set_kwin_block_compositing_with_kwriteconfig(&kwinrc_path) {
        let _ = set_kwin_block_compositing_with_file_patch(&kwinrc_path);
    }

    request_kwin_reconfigure();
}

fn set_kwin_block_compositing_with_kwriteconfig(kwinrc_path: &Path) -> bool {
    let file = kwinrc_path.to_string_lossy().to_string();
    for binary in ["kwriteconfig6", "kwriteconfig5", "kwriteconfig"] {
        let Ok(status) = Command::new(binary)
            .args([
                "--file",
                &file,
                "--group",
                "Compositing",
                "--key",
                "WindowsBlockCompositing",
                "false",
            ])
            .status()
        else {
            continue;
        };

        if status.success() {
            return true;
        }
    }
    false
}

fn set_kwin_block_compositing_with_file_patch(kwinrc_path: &Path) -> bool {
    let content = fs::read_to_string(kwinrc_path).unwrap_or_default();
    let mut changed = false;
    let mut output = Vec::with_capacity(content.lines().count() + 4);
    let mut in_compositing = false;
    let mut saw_compositing_section = false;
    let mut saw_block_key = false;

    for line in content.lines() {
        let trimmed = line.trim();
        let is_section = trimmed.starts_with('[') && trimmed.ends_with(']');

        if is_section {
            if in_compositing && !saw_block_key {
                output.push(String::from("WindowsBlockCompositing=false"));
                changed = true;
            }
            in_compositing = trimmed == "[Compositing]";
            if in_compositing {
                saw_compositing_section = true;
                saw_block_key = false;
            }
            output.push(line.to_string());
            continue;
        }

        if in_compositing && trimmed.starts_with("WindowsBlockCompositing=") {
            saw_block_key = true;
            if trimmed != "WindowsBlockCompositing=false" {
                output.push(String::from("WindowsBlockCompositing=false"));
                changed = true;
            } else {
                output.push(line.to_string());
            }
            continue;
        }

        output.push(line.to_string());
    }

    if saw_compositing_section && in_compositing && !saw_block_key {
        output.push(String::from("WindowsBlockCompositing=false"));
        changed = true;
    }

    if !saw_compositing_section {
        if !output.last().is_some_and(|line| line.is_empty()) {
            output.push(String::new());
        }
        output.push(String::from("[Compositing]"));
        output.push(String::from("WindowsBlockCompositing=false"));
        changed = true;
    }

    if !changed {
        return true;
    }

    if let Some(parent) = kwinrc_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    fs::write(kwinrc_path, format!("{}\n", output.join("\n"))).is_ok()
}

fn request_kwin_reconfigure() {
    let _ = Command::new("dbus-send")
        .args([
            "--session",
            "--type=method_call",
            "--dest=org.kde.KWin",
            "/KWin",
            "org.kde.KWin.reconfigure",
        ])
        .status();
}

fn query_kde_compositing_active() -> Option<bool> {
    let output = Command::new("dbus-send")
        .args([
            "--session",
            "--print-reply",
            "--dest=org.kde.KWin",
            "/Compositor",
            "org.freedesktop.DBus.Properties.Get",
            "string:org.kde.kwin.Compositing",
            "string:active",
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.contains("boolean true") {
        Some(true)
    } else if stdout.contains("boolean false") {
        Some(false)
    } else {
        None
    }
}

fn frame_duration(config: &Config) -> Duration {
    let hz = config.hud.overlay_refresh_rate.clamp(30, 360);
    Duration::from_micros(1_000_000 / hz)
}

impl ApplicationHandler for App {
    fn new_events(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop, cause: StartCause) {
        if let StartCause::ResumeTimeReached { .. } = cause {
            self.refresh_kde_compositor_state();

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
        event: WindowEvent,
    ) {
        while let Ok(message) = self.rx.try_recv() {
            match message {
                Message::GameStatus(status) => self.game_status = status,
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
        let is_gui_window = gui.window().id() == window_id;

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(new_size) => {
                window.resize(new_size);
            }
            WindowEvent::RedrawRequested => {
                event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(
                    self.next_frame_time,
                ));
                // Render only once per frame from the GUI window event.
                if is_gui_window {
                    self.render();
                }
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
