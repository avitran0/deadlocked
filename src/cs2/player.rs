use glam::Vec2;

#[derive(Debug, Default)]
pub struct Target {
    pub pawn: u64,
    pub angle: Vec2,
    pub bone_index: u64,
}

impl Target {
    pub fn reset(&mut self) {
        *self = Target::default();
    }
}
