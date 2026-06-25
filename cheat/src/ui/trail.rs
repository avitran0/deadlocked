use std::time::{Duration, Instant};

use glam::Vec3;

#[derive(Debug)]
#[allow(unused)]
pub struct Trail {
    pub trail: Vec<Vec3>,
    pub last_update: Instant,
}

impl Trail {
    pub const MAX_AGE: Duration = Duration::from_secs(1);
}
