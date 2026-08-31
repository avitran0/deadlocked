use std::{
    str::FromStr,
    time::{Duration, Instant},
};

use axum::{
    Router,
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::Response,
    routing::{any, get},
};
use shared::Data;
use tokio::{net::TcpListener, time::interval};
use utils::log::LoggerOptions;
use uuid::Uuid;

use crate::state::{GameState, ServerState};

mod state;

#[tokio::main]
async fn main() {
    utils::log::init(LoggerOptions::default(), |w, rec| {
        writeln!(w, "[{}] {}", rec.level, rec.args)
    })
    .unwrap();

    let state = ServerState::default();

    let app = Router::new()
        .route("/register", get(register))
        .route("/update", any(update_handler))
        .route("/client", any(client_handler))
        .with_state(state.clone());

    let listener = match TcpListener::bind("0.0.0.0:6346").await {
        Ok(listener) => listener,
        Err(err) => {
            utils::error!("failed to bind port '0.0.0.0:6346': {}", err);
            return;
        }
    };

    tokio::spawn(async move { periodic_clean(state).await });

    if let Err(err) = axum::serve(listener, app).await {
        utils::error!("failed to serve application: {err}");
    }
}

async fn register(State(state): State<ServerState>) -> String {
    let uuid = Uuid::new_v4();
    state.write().games.insert(uuid, GameState::default());
    utils::info!("started session {uuid}");
    format!("{uuid:x}")
}

async fn update_handler(State(state): State<ServerState>, ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(|ws| update(state, ws))
}

async fn update(state: ServerState, mut ws: WebSocket) {
    let Some(uuid) = get_uuid(&mut ws).await else {
        return;
    };

    utils::info!("connected to session {uuid}");

    while let Some(message) = ws.recv().await {
        let message = match message {
            Ok(message) => message,
            Err(err) => {
                utils::error!("failed to receive message: {err}");
                continue;
            }
        };

        if let Message::Close(_) = message {
            break;
        }

        let text = match message.to_text() {
            Ok(text) => text,
            Err(err) => {
                utils::error!("failed to parse message: {err}");
                continue;
            }
        };

        let data: Data = match serde_json::from_str(text) {
            Ok(data) => data,
            Err(err) => {
                utils::error!("failed to parse json: {err}");
                continue;
            }
        };

        state
            .write()
            .games
            .entry(uuid)
            .and_modify(|game| {
                game.data = data;
                game.last_update = Instant::now();
            })
            .or_default();
    }

    state.write().games.remove(&uuid);

    utils::info!("session {uuid} ended");
}

async fn client_handler(State(state): State<ServerState>, ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(|ws| client(state, ws))
}

async fn client(state: ServerState, mut ws: WebSocket) {
    let mut interval = interval(Duration::from_millis(50));

    let Some(uuid) = get_uuid(&mut ws).await else {
        return;
    };

    utils::info!("client connected to {uuid}");

    loop {
        interval.tick().await;

        let json = {
            let state = state.read();
            let Some(game_state) = state.games.get(&uuid) else {
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

    utils::info!("client disconnected from {uuid}");
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

async fn periodic_clean(state: ServerState) {
    let mut interval = interval(Duration::from_secs(5));

    loop {
        interval.tick().await;
        clean(&mut state.write());
    }
}

fn clean(state: &mut state::State) {
    const MAX_AGE: Duration = Duration::from_secs(60);
    state.games.retain(|uuid, game| {
        let valid = game.last_update.elapsed() < MAX_AGE;
        if !valid {
            utils::info!("removed session {uuid} after timeout");
        }
        valid
    });
}
