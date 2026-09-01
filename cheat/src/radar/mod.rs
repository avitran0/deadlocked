use std::{
    net::{TcpStream, ToSocketAddrs},
    ops::Deref,
    str::FromStr,
    sync::Arc,
    time::Duration,
};

use shared::Data;
use tungstenite::{Message, WebSocket, client::IntoClientRequest, stream::MaybeTlsStream};
use utils::{Channel, Mutex};
use uuid::Uuid;

use crate::{
    config::radar::RadarConfig,
    message::{RadarMessage, RadarStatus},
};

pub struct Radar {
    channel: Channel<RadarStatus, RadarMessage>,
    data: Arc<Mutex<Data>>,
    state: Option<RadarState>,
    config: RadarConfig,
}

struct RadarState {
    websocket: WebSocket<MaybeTlsStream<TcpStream>>,
    _uuid: Uuid,
}

impl Radar {
    pub fn new(channel: Channel<RadarStatus, RadarMessage>, data: Arc<Mutex<Data>>) -> Self {
        Self {
            channel,
            data,
            state: None,
            config: RadarConfig::default(),
        }
    }

    fn send_message(&self, message: RadarStatus) {
        if self.channel.send(message).is_err() {
            std::process::exit(1);
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

        if !self.config.enabled {
            self.state = None;
            return false;
        }

        // try to connect to the given url
        if self.state.is_none() {
            self.state = self.establish_connection();
            if self.state.is_none() {
                self.send_message(RadarStatus::FailedToConnect);
            }
        }

        let Some(state) = &mut self.state else {
            return false;
        };

        let Ok(message) = serde_json::to_string(self.data.lock().deref()) else {
            return false;
        };

        if state.websocket.send(Message::Text(message.into())).is_err() {
            self.state = None;
            self.send_message(RadarStatus::FailedToConnect);
            false
        } else {
            true
        }
    }

    fn establish_connection(&self) -> Option<RadarState> {
        let stream = self.connect_tcp()?;
        let uuid = self.register()?;
        self.establish_websocket(stream, uuid)
    }

    fn connect_tcp(&self) -> Option<TcpStream> {
        const TIMEOUT: Duration = Duration::from_secs(5);

        match self.config.url.to_socket_addrs() {
            Ok(mut addresses) => {
                addresses.find_map(
                    |address| match TcpStream::connect_timeout(&address, TIMEOUT) {
                        Ok(stream) => Some(stream),
                        Err(err) => {
                            utils::debug!("failed to connect to {address}: {err}");
                            None
                        }
                    },
                )
            }
            Err(err) => {
                utils::debug!("failed to resolve {}: {err}", self.config.url);
                None
            }
        }
    }

    fn register(&self) -> Option<Uuid> {
        const TIMEOUT: Duration = Duration::from_secs(5);
        let client = ureq::Agent::config_builder()
            .timeout_global(Some(TIMEOUT))
            .build()
            .new_agent();

        let response = match client
            .get(format!("http://{}/register", self.config.url))
            .call()
        {
            Ok(response) => response,
            Err(err) => {
                utils::debug!("failed to register with {}: {err}", self.config.url);
                return None;
            }
        };

        let payload = match response.into_body().read_to_string() {
            Ok(payload) => payload,
            Err(err) => {
                utils::debug!("failed to read registration response: {err}");
                return None;
            }
        };

        match Uuid::from_str(&payload) {
            Ok(uuid) => Some(uuid),
            Err(err) => {
                utils::debug!("invalid session id from server: {err}");
                None
            }
        }
    }

    fn establish_websocket(&self, stream: TcpStream, uuid: Uuid) -> Option<RadarState> {
        const TIMEOUT: Duration = Duration::from_secs(5);

        stream.set_read_timeout(Some(TIMEOUT)).ok()?;
        stream.set_write_timeout(Some(TIMEOUT)).ok()?;

        let (mut websocket, _response) = match tungstenite::client(
            format!("ws://{}/update", self.config.url)
                .into_client_request()
                .ok()?,
            MaybeTlsStream::Plain(stream),
        ) {
            Ok(websocket) => websocket,
            Err(err) => {
                utils::debug!(
                    "failed to connect to websocket server {}: {err}",
                    self.config.url
                );
                return None;
            }
        };

        if let Err(err) = websocket.send(format!("{uuid}").into()) {
            utils::debug!("failed to send session id: {err}");
            return None;
        }

        self.send_message(RadarStatus::Connected(uuid));
        Some(RadarState {
            websocket,
            _uuid: uuid,
        })
    }

    fn process_messages(&mut self) {
        while let Ok(message) = self.channel.try_receive() {
            let url = Self::normalize_url(&message.0.url);
            // reset if url changed
            if url != self.config.url {
                self.send_message(RadarStatus::Disconnected);
                self.state = None;
            }

            self.config = RadarConfig { url, ..message.0 };

            if !self.config.enabled {
                self.send_message(RadarStatus::Disabled);
                self.state = None;
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
