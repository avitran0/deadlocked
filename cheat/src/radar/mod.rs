use std::{net::TcpStream, str::FromStr, sync::Arc};

use shared::Data;
use tungstenite::{WebSocket, stream::MaybeTlsStream};
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
            self.tick();
        }
    }

    fn tick(&mut self) {
        self.process_messages();

        if !self.enabled {
            self.state = None;
        }

        // try to connect to the given url
        if self.state.is_none() {
            self.state = self.establish_connection();
        }

        let Some(state) = &self.state else {
            return;
        };
    }

    fn establish_connection(&self) -> Option<RadarState> {
        let client = ureq::agent();
        let res = match client.get(&self.url).call() {
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

        let (websocket, _response) = match tungstenite::connect(format!("ws://{}", self.url)) {
            Ok(ws) => ws,
            Err(err) => {
                utils::error!("failed to connect to websocket server {}: {err}", self.url);
                return None;
            }
        };

        Some(RadarState { websocket, uuid })
    }

    fn process_messages(&mut self) {
        while let Ok(message) = self.channel.try_receive() {
            match message {
                RadarMessage::Enabled(enabled) => self.enabled = enabled,
                RadarMessage::SetUrl(url) => self.url = url,
            }
        }
    }
}
