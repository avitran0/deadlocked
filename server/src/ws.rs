use std::{str::FromStr, time::Duration};

use axum::{
    Router,
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::{Json, Response},
    routing::{any, get},
};
use tokio::{net::TcpListener, time::interval};
use uuid::Uuid;

use crate::state::ServerState;

pub async fn websocket_server(state: ServerState) {
    utils::info!("starting websocket server on port 6347");

    let app = Router::new()
        .route("/client", any(client_handler))
        .route("/stats", get(stats_handler))
        .with_state(state.clone());

    let listener = match TcpListener::bind("0.0.0.0:6347").await {
        Ok(listener) => listener,
        Err(err) => {
            utils::error!("failed to bind port '0.0.0.0:6347': {}", err);
            return;
        }
    };

    if let Err(err) = axum::serve(listener, app).await {
        utils::error!("failed to serve application: {err}");
    }
}

async fn client_handler(State(state): State<ServerState>, ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(|ws| client(state, ws))
}

async fn client(state: ServerState, mut ws: WebSocket) {
    let connections = state.read().connections.clone();
    let _connection = connections.websocket_guard();
    let mut interval = interval(Duration::from_millis(50));

    let Some(uuid) = get_uuid(&mut ws).await else {
        return;
    };

    utils::info!("client connected to session {uuid}");

    loop {
        interval.tick().await;

        let json = {
            let state = state.read();
            let Some(game_state) = state.sessions.get(&uuid) else {
                break;
            };

            match serde_json::to_string(&game_state.data) {
                Ok(message) => message,
                Err(err) => {
                    utils::warn!("failed to encode data as json: {err}");
                    continue;
                }
            }
        };

        if ws.send(Message::Text(json.into())).await.is_err() {
            break;
        }
    }

    utils::info!("client disconnected from session {uuid}");
}

async fn stats_handler(State(state): State<ServerState>) -> Json<serde_json::Value> {
    let connections = state.read().connections.clone();

    Json(serde_json::json!({
        "tcp_connections": connections.tcp(),
        "websocket_connections": connections.websocket(),
    }))
}

async fn get_uuid(ws: &mut WebSocket) -> Option<Uuid> {
    let uuid_msg = ws.recv().await?;

    let uuid_msg = match uuid_msg {
        Ok(uuid_msg) => uuid_msg,
        Err(err) => {
            utils::error!("failed to receive message: {err}");
            return None;
        }
    };

    let uuid = match uuid_msg.to_text() {
        Ok(uuid) => uuid,
        Err(err) => {
            utils::error!("failed to parse message: {err}");
            return None;
        }
    };

    let uuid = match Uuid::from_str(uuid) {
        Ok(uuid) => uuid,
        Err(err) => {
            utils::error!("invalid uuid: {err}");
            return None;
        }
    };

    Some(uuid)
}
