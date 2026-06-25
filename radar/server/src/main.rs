use std::{net::SocketAddr, time::Duration};

use axum::{
    Router,
    extract::{
        ConnectInfo, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::IntoResponse,
    routing::any,
};
use serde::{Serialize, de::DeserializeOwned};
use shared::data::Data;
use tokio::{net::TcpListener, time::sleep};
use tower_http::services::ServeDir;
use utils::log::LoggerOptions;
use uuid::Uuid;

use crate::{game::Games, message::UuidMessage};

mod game;
mod message;

#[tokio::main]
async fn main() {
    utils::log::init(LoggerOptions::default(), |w, rec| {
        writeln!(w, "[{}] {}", rec.level, rec.args)
    })
    .unwrap();

    let assets_dir = std::env::current_dir().unwrap().join("assets");
    let router = Router::new()
        .fallback_service(ServeDir::new(assets_dir))
        .route("/server", any(server_handler))
        .route("/client", any(client_handler))
        .with_state(Games::default());

    let listener = TcpListener::bind("127.0.0.1:6346").await.unwrap();
    utils::info!("listening on {}", listener.local_addr().unwrap());

    axum::serve(
        listener,
        router.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

async fn server_handler(
    State(state): State<Games>,
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    ws.on_upgrade(|ws| server(state, ws))
}

async fn server(state: Games, mut ws: WebSocket) {
    let uuid = Uuid::new_v4();
    if send_json(&mut ws, &UuidMessage { uuid }).await.is_none() {
        return;
    }

    state.write().await.insert(uuid, Data::default());

    while let Some(data) = recv_json::<Data>(&mut ws).await {
        let mut games = state.write().await;
        let Some(data_mut) = games.get_mut(&uuid) else {
            break;
        };

        *data_mut = data;
    }

    state.write().await.remove(&uuid);
}

async fn client_handler(
    State(state): State<Games>,
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    ws.on_upgrade(|ws| client(state, ws))
}

async fn client(state: Games, mut ws: WebSocket) {
    let Some(uuid) = recv_json::<UuidMessage>(&mut ws).await else {
        return;
    };

    if state.read().await.contains_key(&uuid.uuid) {
        return;
    }

    loop {
        let games = state.read().await;
        let Some(data) = games.get(&uuid.uuid) else {
            return;
        };
        if send_json(&mut ws, data).await.is_none() {
            return;
        }
        drop(games);
        sleep(Duration::from_millis(100)).await;
    }
}

async fn send_json<T: Serialize>(ws: &mut WebSocket, value: &T) -> Option<()> {
    let Ok(json) = serde_json::to_string(value) else {
        return None;
    };

    ws.send(Message::Text(json.into())).await.ok()
}

async fn recv_json<T: DeserializeOwned>(ws: &mut WebSocket) -> Option<T> {
    let Some(Ok(uuid)) = ws.recv().await else {
        return None;
    };

    let Ok(uuid) = uuid.into_text() else {
        return None;
    };

    serde_json::from_str(&uuid).ok()
}
