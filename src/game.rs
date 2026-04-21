use std::{
    sync::Arc,
    thread::sleep,
    time::{Duration, Instant},
};

use utils::{channel::Channel, sync::Mutex};

use crate::{
    config::{CONFIG_PATH, Config, DEFAULT_CONFIG_NAME, SLEEP_DURATION, parse_config},
    cs2::CS2,
    data::Data,
    message::{GameMessage, GameStatus, UiMessage},
    os::mouse::Mouse,
};

pub struct GameManager {
    channel: Channel<UiMessage, GameMessage>,
    data: Arc<Mutex<Data>>,
    config: Config,
    mouse: Mouse,
    cs2: CS2,
}

impl GameManager {
    pub fn new(channel: Channel<UiMessage, GameMessage>, data: Arc<Mutex<Data>>) -> Self {
        let mouse = match Mouse::open() {
            Ok(mouse) => mouse,
            Err(err) => {
                utils::error!("error creating uinput device: {err}");
                utils::error!("uinput kernel module is not loaded, or user is not in input group.");
                std::process::exit(1);
            }
        };

        let mut game = Self {
            channel,
            data,
            config: Config::default(),
            mouse,
            cs2: CS2::new(),
        };

        let config_path = CONFIG_PATH.join(DEFAULT_CONFIG_NAME);
        if config_path.exists() {
            game.config = parse_config(&config_path);
        }

        game
    }

    fn send_message(&self, message: UiMessage) {
        if self.channel.send(message).is_err() {
            std::process::exit(1);
        }
    }

    pub fn run(&mut self) {
        self.send_message(UiMessage(GameStatus::NotStarted));
        let mut previous_status = GameStatus::NotStarted;
        let mut next_data_tick = Instant::now();
        loop {
            let start = Instant::now();
            while let Ok(message) = self.channel.try_receive() {
                self.config = *message.0;
            }

            let mut is_valid = self.cs2.is_valid();
            if !is_valid {
                if previous_status == GameStatus::Working {
                    self.send_message(UiMessage(GameStatus::NotStarted));
                    previous_status = GameStatus::NotStarted;
                }
                self.cs2.setup();
                is_valid = self.cs2.is_valid();
            }

            if is_valid {
                if previous_status == GameStatus::NotStarted {
                    self.send_message(UiMessage(GameStatus::Working));
                    previous_status = GameStatus::Working;
                }
                self.cs2.run(&self.config, &mut self.mouse);
                let now = Instant::now();
                if now >= next_data_tick {
                    let mut data = self.data.lock();
                    self.cs2.data(&self.config, &mut data);
                    next_data_tick = now + data_loop_duration(&self.config);
                }
            } else {
                *self.data.lock() = Data::default();
                next_data_tick = Instant::now();
            }

            if is_valid {
                let elapsed = start.elapsed();
                if elapsed < self.loop_duration() {
                    sleep(self.loop_duration() - elapsed);
                } else {
                    utils::debug!(
                        "game loop took {} ms (max {} ms)",
                        elapsed.as_millis(),
                        self.loop_duration().as_millis()
                    );
                }
            } else {
                sleep(SLEEP_DURATION);
            }
        }
    }

    fn loop_duration(&self) -> Duration {
        Duration::from_secs_f32(1.0 / self.config.fps as f32)
    }
}

fn data_loop_duration(config: &Config) -> Duration {
    let hz = config.hud.data_hz();
    Duration::from_micros(1_000_000 / hz)
}
