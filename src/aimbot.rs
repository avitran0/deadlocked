use std::{sync::mpsc, thread::sleep, time::Instant};

use log::{info, warn};

use crate::{
    config::{parse_config, AimbotStatus, LOOP_DURATION},
    config::{Config, SLEEP_DURATION},
    cs2::CS2,
    message::Message,
    mouse::DeviceStatus,
    mouse::Mouse,
};

pub trait Aimbot: std::fmt::Debug {
    fn is_valid(&self) -> bool;
    fn setup(&mut self);
    fn run(&mut self, config: &Config, mouse: &mut Mouse);
}

pub struct AimbotManager {
    tx: mpsc::Sender<Message>,
    rx: mpsc::Receiver<Message>,
    config: Config,
    mouse: Mouse,
    aimbot: CS2,
}

impl AimbotManager {
    pub fn new(tx: mpsc::Sender<Message>, rx: mpsc::Receiver<Message>) -> Self {
        let mouse = Mouse::open();

        let config = parse_config();
        let mut aimbot = Self {
            tx,
            rx,
            config,
            mouse,
            aimbot: CS2::new(),
        };

        aimbot.send_message(Message::MouseStatus(aimbot.mouse.status.clone()));

        aimbot
    }

    fn send_message(&mut self, message: Message) {
        self.tx.send(message).unwrap();
    }

    pub fn run(&mut self) {
        self.send_message(Message::Status(AimbotStatus::GameNotStarted));
        let mut previous_status = AimbotStatus::GameNotStarted;
        loop {
            let start = Instant::now();
            while let Ok(message) = self.rx.try_recv() {
                self.parse_message(message);
            }

            let mut mouse_valid = self.mouse.is_valid();
            if !mouse_valid || self.mouse.status == DeviceStatus::NotFound {
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
        self.send_message(Message::MouseStatus(DeviceStatus::Disconnected));
        info!("mouse disconnected");
        self.mouse.status = DeviceStatus::Disconnected;
        let mouse = Mouse::open();
        if let DeviceStatus::Working(_) = mouse.status {
            info!("mouse reconnected");
            mouse_valid = true;
        }
        self.send_message(Message::MouseStatus(mouse.status.clone()));
        self.mouse = mouse;
        mouse_valid
    }
}
