use std::time::Instant;

use shared::{Data, MAX_FRAME_SIZE, PROTO_VERSION};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
use uuid::Uuid;

use crate::state::{ServerState, SessionState};

pub async fn tcp_server(state: ServerState) {
    utils::info!("starting tcp server on port 6346");

    let listener = match TcpListener::bind("0.0.0.0:6346").await {
        Ok(listener) => listener,
        Err(err) => {
            utils::error!("failed to bind port '0.0.0.0:6346': {}", err);
            return;
        }
    };

    loop {
        let (stream, _addr) = match listener.accept().await {
            Ok(tcp) => tcp,
            Err(err) => {
                utils::warn!("failed to accept connection: {err}");
                continue;
            }
        };

        let connection_state = state.clone();
        tokio::spawn(async move { tcp_handler(stream, connection_state).await });
    }
}

async fn tcp_handler(mut stream: TcpStream, state: ServerState) {
    let Ok(proto_version) = stream.read_u32().await else {
        utils::error!("failed to receive protocol version");
        return;
    };

    if proto_version != PROTO_VERSION {
        utils::error!("protocol version is {PROTO_VERSION}, received {proto_version}");
        return;
    }

    let Ok(uuid_value) = stream.read_u128().await else {
        utils::error!("failed to receive session uuid");
        return;
    };

    if stream.write_u128(uuid_value).await.is_err() {
        utils::error!("failed to send handshake");
        return;
    }

    let uuid = Uuid::from_u128(uuid_value);
    state.write().sessions.insert(uuid, SessionState::default());
    utils::info!("connected to session {uuid}");

    while let Ok(frame_length) = stream.read_u32().await {
        if frame_length > MAX_FRAME_SIZE {
            utils::error!(
                "received oversized data frame from session {uuid}: {frame_length} bytes"
            );
            break;
        }

        let mut buf = vec![0u8; frame_length as usize];
        if stream.read_exact(&mut buf).await.is_err() {
            break;
        }

        let data: Data = match postcard::from_bytes(&buf) {
            Ok(data) => data,
            Err(err) => {
                utils::error!("failed to deserialize data: {err}");
                continue;
            }
        };

        state.write().sessions.entry(uuid).and_modify(|session| {
            session.data = data;
            session.last_update = Instant::now();
        });
    }

    state.write().sessions.remove(&uuid);
    utils::info!("removed session {uuid}");
}
