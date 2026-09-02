use std::{collections::HashMap, sync::Arc, time::Instant};

use shared::Data;
use utils::RwLock;
use uuid::Uuid;

pub type ServerState = Arc<RwLock<State>>;

#[derive(Default)]
pub struct State {
    pub sessions: HashMap<Uuid, SessionState>,
}

pub struct SessionState {
    pub data: Data,
    pub last_update: Instant,
}

impl Default for SessionState {
    fn default() -> Self {
        Self {
            data: Data::default(),
            last_update: Instant::now(),
        }
    }
}
