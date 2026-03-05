use std::{sync::Arc, thread::sleep, time::Instant};

use crossbeam::channel::{Receiver, Sender};
use utils::{log, sync::Mutex};

use crate::{
    config::{
        CONFIG_PATH, Config, DEFAULT_CONFIG_NAME, LOOP_DURATION, SLEEP_DURATION, parse_config,
    },
    cs2::CS2,
    data::Data,
    message::{Envelope, GameStatus, Message, Target},
    os::mouse::Mouse,
    ui::grenades::GrenadeList,
};

pub trait Game: std::fmt::Debug {
    fn is_valid(&self) -> bool;
    fn setup(&mut self);
    fn run(&mut self, config: &Config, mouse: &mut Mouse);
    fn data(&self, config: &Config, data: &mut Data);
}

pub struct GameManager {
    tx: Sender<Envelope>,
    rx: Receiver<Message>,
    data: Arc<Mutex<Data>>,
    config: Config,
    mouse: Mouse,
    game: CS2,
}

impl GameManager {
    pub fn new(
        tx: Sender<Envelope>,
        rx: Receiver<Message>,
        data: Arc<Mutex<Data>>,
        grenades: Arc<Mutex<GrenadeList>>,
    ) -> Self {
        let mouse = match Mouse::open() {
            Ok(mouse) => mouse,
            Err(err) => {
                log::error!("error creating uinput device: {err}");
                log::error!("uinput kernel module is not loaded, or user is not in input group.");
                std::process::exit(1);
            }
        };

        let mut game = Self {
            tx,
            rx,
            data,
            config: Config::default(),
            mouse,
            game: CS2::new(grenades),
        };

        let config_path = CONFIG_PATH.join(DEFAULT_CONFIG_NAME);
        if config_path.exists() {
            game.config = parse_config(&config_path);
        }

        game
    }

    fn send_game_message(&self, message: Message) {
        let envelope = Envelope {
            target: Target::Gui,
            message,
        };
        if self.tx.send(envelope).is_err() {
            std::process::exit(1);
        }
    }

    pub fn run(&mut self) {
        self.send_game_message(Message::GameStatus(GameStatus::NotStarted));
        let mut previous_status = GameStatus::NotStarted;
        let mut next_data_tick = Instant::now();
        loop {
            let start = Instant::now();
            while let Ok(message) = self.rx.try_recv() {
                self.parse_message(message);
            }

            let mut is_valid = self.game.is_valid();
            if !is_valid {
                if previous_status == GameStatus::Working {
                    self.send_game_message(Message::GameStatus(GameStatus::NotStarted));
                    previous_status = GameStatus::NotStarted;
                }
                self.game.setup();
                is_valid = self.game.is_valid();
            }

            if is_valid {
                if previous_status == GameStatus::NotStarted {
                    self.send_game_message(Message::GameStatus(GameStatus::Working));
                    previous_status = GameStatus::Working;
                }
                self.game.run(&self.config, &mut self.mouse);
                let now = Instant::now();
                if now >= next_data_tick {
                    let mut data = self.data.lock();
                    self.game.data(&self.config, &mut data);
                    next_data_tick = now + data_loop_duration(&self.config);
                }
            } else {
                *self.data.lock() = Data::default();
                next_data_tick = Instant::now();
            }

            if is_valid {
                let elapsed = start.elapsed();
                if elapsed < LOOP_DURATION {
                    sleep(LOOP_DURATION - elapsed);
                } else {
                    log::debug!(
                        "game loop took {} ms (max {} ms)",
                        elapsed.as_millis(),
                        LOOP_DURATION.as_millis()
                    );
                    sleep(LOOP_DURATION);
                }
            } else {
                sleep(SLEEP_DURATION);
            }
        }
    }

    fn parse_message(&mut self, message: Message) {
        if let Message::Config(config) = message {
            self.config = *config;
        }
    }
}

fn data_loop_duration(config: &Config) -> std::time::Duration {
    let hz = config.hud.data_refresh_rate.clamp(20, 240);
    std::time::Duration::from_micros(1_000_000 / hz)
}
