use std::{fs::File, sync::mpsc, thread::sleep, time::Instant};

use log::warn;

use crate::{
    config::{Config, DEBUG_WITHOUT_MOUSE, SLEEP_DURATION},
    cs2::CS2,
    message::Game,
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
}

pub struct AimbotManager {
    tx: mpsc::Sender<AimbotMessage>,
    rx: mpsc::Receiver<AimbotMessage>,
    config: Config,
    mouse: File,
    mouse_status: MouseStatus,
    aimbot: Box<dyn Aimbot>,
}

impl AimbotManager {
    pub fn new(tx_gui: mpsc::Sender<AimbotMessage>, rx: mpsc::Receiver<AimbotMessage>) -> Self {
        let (mouse, status) = open_mouse();

        let config = parse_config();
        let aimbot = Box::new(match config.current_game {
            Game::CS2 => CS2::new(),
            Game::Deadlock => CS2::new(),
        });
        let mut aimbot = Self {
            tx: tx_gui,
            rx,
            config,
            mouse,
            mouse_status: status.clone(),
            aimbot,
        };

        aimbot.send_message(AimbotMessage::MouseStatus(status));

        aimbot
    }

    fn send_message(&mut self, message: AimbotMessage) {
        let _ = self.tx.send(message);
    }

    pub fn run(&mut self) {
        self.send_message(AimbotMessage::Status(AimbotStatus::GameNotStarted));
        let mut previous_status = AimbotStatus::GameNotStarted;
        loop {
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
            }

            if self.aimbot.is_valid() && mouse_valid {
                let elapsed = start.elapsed();
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
            AimbotMessage::ConfigEnableAimbot(aimbot) => config.enabled = aimbot,
            AimbotMessage::ConfigHotkey(hotkey) => config.hotkey = hotkey,
            AimbotMessage::ConfigStartBullet(start_bullet) => config.start_bullet = start_bullet,
            AimbotMessage::ConfigAimLock(aim_lock) => config.aim_lock = aim_lock,
            AimbotMessage::ConfigVisibilityCheck(visibility_check) => {
                config.visibility_check = visibility_check
            }
            AimbotMessage::ConfigFOV(fov) => config.fov = fov,
            AimbotMessage::ConfigSmooth(smooth) => config.smooth = smooth,
            AimbotMessage::ConfigMultibone(multibone) => config.multibone = multibone,
            AimbotMessage::ConfigEnableRCS(rcs) => config.rcs = rcs,
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
