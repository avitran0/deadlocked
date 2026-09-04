use std::time::Duration;

use tokio::time::interval;
use utils::log::LoggerOptions;

use crate::{state::ServerState, tcp::tcp_server, ws::websocket_server};

mod state;
mod tcp;
mod ws;

#[tokio::main]
async fn main() {
    utils::log::init(LoggerOptions::default(), |w, rec| {
        writeln!(w, "[{}] {}", rec.level, rec.args)
    })
    .unwrap();

    let state = ServerState::default();
    let state_tcp = state.clone();
    let state_ws = state.clone();

    tokio::join!(
        periodic_clean(state),
        tcp_server(state_tcp),
        websocket_server(state_ws),
    );
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
    state.sessions.retain(|uuid, game| {
        let valid = game.last_update.elapsed() < MAX_AGE;
        if !valid {
            utils::info!("removed session {uuid} after timeout");
        }
        valid
    });
}
