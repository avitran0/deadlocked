use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Instant,
};

use shared::Data;
use utils::RwLock;
use uuid::Uuid;

pub type ServerState = Arc<RwLock<State>>;

#[derive(Default)]
pub struct State {
    pub sessions: HashMap<Uuid, SessionState>,
    pub connections: Arc<ConnectionCounts>,
}

#[derive(Default)]
pub struct ConnectionCounts {
    tcp: AtomicUsize,
    websocket: AtomicUsize,
}

impl ConnectionCounts {
    pub fn tcp(&self) -> usize {
        self.tcp.load(Ordering::Relaxed)
    }

    pub fn websocket(&self) -> usize {
        self.websocket.load(Ordering::Relaxed)
    }

    pub fn tcp_guard(self: &Arc<Self>) -> ConnectionGuard {
        self.tcp.fetch_add(1, Ordering::Relaxed);
        ConnectionGuard {
            count: Arc::clone(self),
            kind: ConnectionKind::Tcp,
        }
    }

    pub fn websocket_guard(self: &Arc<Self>) -> ConnectionGuard {
        self.websocket.fetch_add(1, Ordering::Relaxed);
        ConnectionGuard {
            count: Arc::clone(self),
            kind: ConnectionKind::Websocket,
        }
    }
}

enum ConnectionKind {
    Tcp,
    Websocket,
}

pub struct ConnectionGuard {
    count: Arc<ConnectionCounts>,
    kind: ConnectionKind,
}

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        match self.kind {
            ConnectionKind::Tcp => self.count.tcp.fetch_sub(1, Ordering::Relaxed),
            ConnectionKind::Websocket => self.count.websocket.fetch_sub(1, Ordering::Relaxed),
        };
    }
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
