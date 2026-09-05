use std::time::Instant;

use shared::{Data, HEARTBEAT, MAX_FRAME_SIZE, PROTO_VERSION};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    time::{Duration, timeout},
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
    const INACTIVITY_TIMEOUT: Duration = Duration::from_secs(10);

    let connections = state.read().connections.clone();
    let _connection = connections.tcp_guard();

    let proto_version = match timeout(INACTIVITY_TIMEOUT, stream.read_u32()).await {
        Ok(Ok(proto_version)) => proto_version,
        Ok(Err(_)) => {
            utils::error!("failed to receive protocol version");
            return;
        }
        Err(_) => {
            utils::warn!("timed out waiting for protocol version");
            return;
        }
    };

    if proto_version != PROTO_VERSION {
        utils::error!("protocol version is {PROTO_VERSION}, received {proto_version}");
        return;
    }

    let uuid_value = match timeout(INACTIVITY_TIMEOUT, stream.read_u128()).await {
        Ok(Ok(uuid_value)) => uuid_value,
        Ok(Err(_)) => {
            utils::error!("failed to receive session uuid");
            return;
        }
        Err(_) => {
            utils::warn!("timed out waiting for session uuid");
            return;
        }
    };

    if stream.write_u128(uuid_value).await.is_err() {
        utils::error!("failed to send handshake");
        return;
    }

    let uuid = Uuid::from_u128(uuid_value);
    state.write().sessions.insert(uuid, SessionState::default());
    utils::info!("connected to session {uuid}");

    loop {
        let frame_length = match timeout(INACTIVITY_TIMEOUT, stream.read_u32()).await {
            Ok(Ok(frame_length)) => frame_length,
            Ok(Err(_)) => break,
            Err(_) => {
                utils::warn!("removed session {uuid} after inactivity timeout");
                break;
            }
        };

        if frame_length == HEARTBEAT {
            continue;
        }

        if frame_length > MAX_FRAME_SIZE {
            utils::error!(
                "received oversized data frame from session {uuid}: {frame_length} bytes"
            );
            break;
        }

        let mut buf = vec![0u8; frame_length as usize];
        if timeout(INACTIVITY_TIMEOUT, stream.read_exact(&mut buf))
            .await
            .is_err()
        {
            utils::warn!("removed session {uuid} after incomplete frame timeout");
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
