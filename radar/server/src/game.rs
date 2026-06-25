use std::{collections::HashMap, sync::Arc};

use shared::data::Data;
use tokio::sync::RwLock;
use uuid::Uuid;

pub type Games = Arc<RwLock<HashMap<Uuid, Data>>>;
