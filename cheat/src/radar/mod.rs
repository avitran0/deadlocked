use std::{
    io::Write,
    net::{TcpStream, ToSocketAddrs},
    ops::Deref,
    sync::Arc,
    time::{Duration, Instant},
};

use shared::{Data, HEARTBEAT, MAX_FRAME_SIZE, PROTO_VERSION};
use utils::{
    Channel, Mutex,
    io::{Endian, EndianReaderWriter},
};
use uuid::Uuid;

use crate::{
    config::radar::RadarConfig,
    message::{RadarMessage, RadarStatus},
};

pub struct Radar {
    channel: Channel<RadarStatus, RadarMessage>,
    data: Arc<Mutex<Data>>,
    uuid: Option<Uuid>,
    state: Option<RadarState>,
    config: RadarConfig,
}

struct RadarState {
    stream: EndianReaderWriter<TcpStream>,
    last_heartbeat: Instant,
}

impl Radar {
    pub fn new(channel: Channel<RadarStatus, RadarMessage>, data: Arc<Mutex<Data>>) -> Self {
        Self {
            channel,
            data,
            uuid: None,
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

        if self.uuid.is_none() {
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

        self.send_packet()
    }

    fn send_packet(&mut self) -> bool {
        let Some(state) = &mut self.state else {
            return false;
        };

        let Ok(message) = postcard::to_stdvec(self.data.lock().deref()) else {
            return false;
        };

        if message.len() > MAX_FRAME_SIZE as usize {
            utils::error!("serialized radar data exceeds maximum frame size");
            self.state = None;
            self.send_message(RadarStatus::FailedToConnect);
            return false;
        }

        if state.stream.write_u32(message.len() as u32).is_err()
            || state.stream.write_all(&message).is_err()
        {
            self.state = None;
            self.send_message(RadarStatus::FailedToConnect);
            return false;
        }

        const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
        if state.last_heartbeat.elapsed() >= HEARTBEAT_INTERVAL {
            if state.stream.write_u32(HEARTBEAT).is_err() {
                self.state = None;
                self.send_message(RadarStatus::FailedToConnect);
                return false;
            }
            state.last_heartbeat = Instant::now();
        }

        true
    }

    fn establish_connection(&self) -> Option<RadarState> {
        let stream = self.connect_tcp()?;
        self.connect_server(stream)
    }

    fn connect_tcp(&self) -> Option<TcpStream> {
        const TIMEOUT: Duration = Duration::from_secs(5);

        let mut addresses = match (self.config.url.as_ref(), 6346).to_socket_addrs() {
            Ok(addresses) => addresses,
            Err(err) => {
                utils::debug!("failed to resolve {}: {err}", self.config.url);
                return None;
            }
        };

        addresses.find_map(|address| {
            let stream = match TcpStream::connect_timeout(&address, TIMEOUT) {
                Ok(stream) => stream,
                Err(err) => {
                    utils::debug!("failed to connect to {address}: {err}");
                    return None;
                }
            };

            stream.set_read_timeout(Some(TIMEOUT)).ok()?;
            stream.set_write_timeout(Some(TIMEOUT)).ok()?;

            Some(stream)
        })
    }

    fn connect_server(&self, stream: TcpStream) -> Option<RadarState> {
        let mut stream = EndianReaderWriter::new(stream, Endian::Big);

        let uuid = self.uuid?;
        stream.write_u32(PROTO_VERSION).ok()?;
        stream.write_u128(uuid.as_u128()).ok()?;
        let received_uuid = Uuid::from_u128(stream.read_u128().ok()?);

        if received_uuid != uuid {
            return None;
        }

        self.send_message(RadarStatus::Connected);
        Some(RadarState {
            stream,
            last_heartbeat: Instant::now(),
        })
    }

    fn process_messages(&mut self) {
        while let Ok(message) = self.channel.try_receive() {
            match message {
                RadarMessage::Config { config, uuid } => {
                    let url = Self::normalize_url(&config.url);
                    // reset if the URL or session UUID changed
                    if url != self.config.url || self.uuid != Some(uuid) {
                        self.send_message(RadarStatus::Disconnected);
                        self.state = None;
                    }

                    self.uuid = Some(uuid);
                    self.config = RadarConfig { url, ..config };

                    if !self.config.enabled {
                        self.send_message(RadarStatus::Disabled);
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
