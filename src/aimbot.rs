use std::{fs::File, sync::mpsc, thread::sleep, time::Instant};

use log::{info, warn};

use crate::{
    config::{Config, SLEEP_DURATION},
    cs2::CS2,
    mouse::{mouse_valid, MouseStatus},
};

use crate::{
    config::{parse_config, AimbotStatus, LOOP_DURATION},
    message::Message,
    mouse::open_mouse,
};

pub trait Aimbot: std::fmt::Debug {
    fn is_valid(&self) -> bool;
    fn setup(&mut self);
    fn run(&mut self, config: &Config, mouse: &mut File);
}

pub struct AimbotManager {
    tx: mpsc::Sender<Message>,
    rx: mpsc::Receiver<Message>,
    config: Config,
    mouse: File,
    mouse_status: MouseStatus,
    aimbot: CS2,
}

impl AimbotManager {
    pub fn new(tx: mpsc::Sender<Message>, rx: mpsc::Receiver<Message>) -> Self {
        let (mouse, status) = open_mouse();

        let config = parse_config();
        let mut aimbot = Self {
            tx,
            rx,
            config,
            mouse,
            mouse_status: status.clone(),
            aimbot: CS2::new(),
        };

        aimbot.send_message(Message::MouseStatus(status));

        aimbot
    }

    fn send_message(&mut self, message: Message) {
        let _ = self.tx.send(message);
    }

    pub fn run(&mut self) {
        self.send_message(Message::Status(AimbotStatus::GameNotStarted));
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

            if !self.aimbot.is_valid() {
                if previous_status == AimbotStatus::Working {
                    self.send_message(Message::Status(AimbotStatus::GameNotStarted));
                    previous_status = AimbotStatus::GameNotStarted;
                }
                self.aimbot.setup();
            }

            if mouse_valid && self.aimbot.is_valid() {
                if previous_status == AimbotStatus::GameNotStarted {
                    self.send_message(Message::Status(AimbotStatus::Working));
                    previous_status = AimbotStatus::Working;
                }
                self.aimbot.run(&self.config, &mut self.mouse);
            }

            if self.aimbot.is_valid() && mouse_valid {
                let elapsed = start.elapsed();
                self.send_message(Message::FrameTime(elapsed.as_micros() as f64));
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

    fn parse_message(&mut self, message: Message) {
        if let Message::Config(config) = message {
            self.config = config
        }
    }

    fn find_mouse(&mut self) -> bool {
        let mut mouse_valid = false;
        self.send_message(Message::MouseStatus(MouseStatus::Disconnected));
        info!("mouse disconnected");
        self.mouse_status = MouseStatus::Disconnected;
        let (mouse, status) = open_mouse();
        if let MouseStatus::Working(_) = status {
            info!("mouse reconnected");
            mouse_valid = true;
        }
        self.send_message(Message::MouseStatus(status.clone()));
        self.mouse_status = status;
        self.mouse = mouse;
        mouse_valid
    }
}
