use std::{fs::File, sync::mpsc, thread::sleep, time::Instant};

use glam::{IVec4, Mat4};
use log::warn;

use crate::{
    config::{Config, DEBUG_WITHOUT_MOUSE, SLEEP_DURATION},
    cs2::CS2,
    message::{EntityInfo, Game, PlayerInfo, VisualsMessage},
    mouse::{mouse_valid, MouseStatus},
};

use crate::{
    config::{parse_config, AimbotStatus, LOOP_DURATION},
    message::AimbotMessage,
    mouse::open_mouse,
};

pub trait Aimbot: std::fmt::Debug {
    fn is_valid(&self) -> bool;
    fn setup(&mut self);
    fn run(&mut self, config: &Config, mouse: &mut File);
    fn get_player_info(&mut self) -> Vec<PlayerInfo>;
    fn get_entity_info(&mut self) -> Vec<EntityInfo>;
    fn get_view_matrix(&mut self) -> Mat4;
    fn get_window_size(&mut self) -> IVec4;
}

pub struct AimbotManager {
    tx_gui: mpsc::Sender<AimbotMessage>,
    tx_visuals: mpsc::Sender<VisualsMessage>,
    rx: mpsc::Receiver<AimbotMessage>,
    config: Config,
    mouse: File,
    mouse_status: MouseStatus,
    aimbot: Box<dyn Aimbot>,
    should_quit: bool,
}

impl AimbotManager {
    pub fn new(
        tx_gui: mpsc::Sender<AimbotMessage>,
        tx_visuals: mpsc::Sender<VisualsMessage>,
        rx: mpsc::Receiver<AimbotMessage>,
    ) -> Self {
        let (mouse, status) = open_mouse();

        let config = parse_config();
        let aimbot = Box::new(match config.current_game {
            Game::CS2 => CS2::new(),
            Game::Deadlock => CS2::new(),
        });
        let mut aimbot = Self {
            tx_gui,
            tx_visuals,
            rx,
            config,
            mouse,
            mouse_status: status.clone(),
            aimbot,
            should_quit: false,
        };

        aimbot.send_message(AimbotMessage::MouseStatus(status));

        aimbot
    }

    fn send_message(&mut self, message: AimbotMessage) {
        let _ = self.tx_gui.send(message);
    }

    fn send_visuals_message(&mut self, message: VisualsMessage) {
        let _ = self.tx_visuals.send(message);
    }

    pub fn run(&mut self) {
        self.send_message(AimbotMessage::Status(AimbotStatus::GameNotStarted));
        let mut previous_status = AimbotStatus::GameNotStarted;
        loop {
            if self.should_quit {
                break;
            }
            let start = Instant::now();
            while let Ok(message) = self.rx.try_recv() {
                self.parse_message(message);
            }

            let mut mouse_valid = mouse_valid(&mut self.mouse);
            if !mouse_valid || self.mouse_status == MouseStatus::NoMouseFound {
                mouse_valid = self.find_mouse();
            }
            // todo: refactor to an if let chain when that is stabilized
            if let MouseStatus::Working(path) = &self.mouse_status {
                if path == "/dev/null" && !DEBUG_WITHOUT_MOUSE {
                    mouse_valid = self.find_mouse();
                }
            }

            if !self.aimbot.is_valid() {
                if previous_status == AimbotStatus::Working {
                    self.send_message(AimbotMessage::Status(AimbotStatus::GameNotStarted));
                    previous_status = AimbotStatus::GameNotStarted;
                }
                self.aimbot.setup();
            }
            if mouse_valid && self.aimbot.is_valid() {
                if previous_status == AimbotStatus::GameNotStarted {
                    self.send_message(AimbotMessage::Status(AimbotStatus::Working));
                    previous_status = AimbotStatus::Working;
                }
                self.aimbot.run(&self.config, &mut self.mouse);
                let players = self.aimbot.get_player_info();
                self.send_visuals_message(VisualsMessage::PlayerInfo(players));
                let entities = self.aimbot.get_entity_info();
                self.send_visuals_message(VisualsMessage::EntityInfo(entities));
                let view_matrix = self.aimbot.get_view_matrix();
                self.send_visuals_message(VisualsMessage::ViewMatrix(view_matrix));
                let window_info = self.aimbot.get_window_size();
                self.send_visuals_message(VisualsMessage::WindowSize(window_info));
            }

            if self.aimbot.is_valid() && mouse_valid {
                let elapsed = start.elapsed();
                self.send_message(AimbotMessage::FrameTime(elapsed.as_millis() as f64));
                if elapsed < LOOP_DURATION {
                    sleep(LOOP_DURATION - elapsed);
                } else {
                    warn!("aimbot loop took {}ms", elapsed.as_millis());
                    sleep(LOOP_DURATION);
                }
            } else {
                sleep(SLEEP_DURATION);
            }
        }
    }

    fn parse_message(&mut self, message: AimbotMessage) {
        if let AimbotMessage::ChangeGame(game) = message {
            self.config.current_game = game;
            return;
        }
        let config = self
            .config
            .games
            .get_mut(&self.config.current_game)
            .unwrap();
        match message {
            AimbotMessage::ConfigEnableAimbot(aimbot) => config.aimbot.enabled = aimbot,
            AimbotMessage::ConfigHotkey(hotkey) => config.aimbot.hotkey = hotkey,
            AimbotMessage::ConfigStartBullet(start_bullet) => {
                config.aimbot.start_bullet = start_bullet
            }
            AimbotMessage::ConfigAimLock(aim_lock) => config.aimbot.aim_lock = aim_lock,
            AimbotMessage::ConfigVisibilityCheck(visibility_check) => {
                config.aimbot.visibility_check = visibility_check
            }
            AimbotMessage::ConfigFOV(fov) => config.aimbot.fov = fov,
            AimbotMessage::ConfigSmooth(smooth) => config.aimbot.smooth = smooth,
            AimbotMessage::ConfigMultibone(multibone) => config.aimbot.multibone = multibone,
            AimbotMessage::ConfigEnableRCS(rcs) => config.aimbot.rcs = rcs,
            AimbotMessage::Quit => self.should_quit = true,
            _ => {}
        }
    }

    fn find_mouse(&mut self) -> bool {
        let mut mouse_valid = false;
        self.send_message(AimbotMessage::MouseStatus(MouseStatus::Disconnected));
        self.mouse_status = MouseStatus::Disconnected;
        let (mouse, status) = open_mouse();
        if let MouseStatus::Working(path) = &status {
            if path != "/dev/null" {
                mouse_valid = true;
            }
        }
        self.send_message(AimbotMessage::MouseStatus(status.clone()));
        self.mouse_status = status;
        self.mouse = mouse;
        mouse_valid
    }
}
