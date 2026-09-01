use std::{net::TcpStream, ops::Deref, str::FromStr, sync::Arc, time::Duration};

use shared::Data;
use tungstenite::{Message, WebSocket, stream::MaybeTlsStream};
use utils::{Channel, Mutex};
use uuid::Uuid;

use crate::message::RadarMessage;

pub struct Radar {
    channel: Channel<(), RadarMessage>,
    data: Arc<Mutex<Data>>,
    state: Option<RadarState>,
    enabled: bool,
    url: String,
}

struct RadarState {
    websocket: WebSocket<MaybeTlsStream<TcpStream>>,
    uuid: Uuid,
}

impl Radar {
    pub fn new(channel: Channel<(), RadarMessage>, data: Arc<Mutex<Data>>) -> Self {
        Self {
            channel,
            data,
            state: None,
            enabled: true,
            url: String::new(),
        }
    }

    pub fn run(mut self) {
        loop {
            if self.tick() {
                std::thread::sleep(Duration::from_millis(50));
            } else {
                std::thread::sleep(Duration::from_secs(1));
            }
        }
    }

    fn tick(&mut self) -> bool {
        self.process_messages();

        if !self.enabled {
            self.state = None;
            return false;
        }

        // try to connect to the given url
        if self.state.is_none() {
            self.state = self.establish_connection();
        }

        let Some(state) = &mut self.state else {
            return false;
        };

        let Ok(message) = serde_json::to_string(self.data.lock().deref()) else {
            return false;
        };

        if state.websocket.send(Message::Text(message.into())).is_err() {
            self.state = None;
            false
        } else {
            true
        }
    }

    fn establish_connection(&self) -> Option<RadarState> {
        let client = ureq::agent();
        let res = match client.get(format!("http://{}/register", self.url)).call() {
            Ok(res) => res,
            Err(err) => {
                utils::error!("failed to connect to {}: {err}", &self.url);
                return None;
            }
        };

        let res_str = match res.into_body().read_to_string() {
            Ok(res) => res,
            Err(err) => {
                utils::error!("invalid server payload: {err}");
                return None;
            }
        };

        let uuid = match Uuid::from_str(&res_str) {
            Ok(uuid) => uuid,
            Err(err) => {
                utils::error!("invalid uuid: {err}");
                return None;
            }
        };

        let (mut websocket, _response) =
            match tungstenite::connect(format!("ws://{}/update", self.url)) {
                Ok(ws) => ws,
                Err(err) => {
                    utils::error!("failed to connect to websocket server {}: {err}", self.url);
                    return None;
                }
            };

        // need to send the uuid once
        websocket.send(format!("{uuid}").into()).ok()?;

        Some(RadarState { websocket, uuid })
    }

    fn process_messages(&mut self) {
        while let Ok(message) = self.channel.try_receive() {
            match message {
                RadarMessage::Enabled(enabled) => self.enabled = enabled,
                RadarMessage::SetUrl(url) => {
                    let url = Self::normalize_url(&url);

                    if self.url != url {
                        self.url = url;
                        self.state = None;
                    }
                }
            }
        }
    }

    fn normalize_url(url: &str) -> String {
        let url = url.trim();
        let url = url
            .strip_prefix("http://")
            .or_else(|| url.strip_prefix("https://"))
            .unwrap_or(url);
        url.trim_end_matches('/').to_owned()
    }
}
