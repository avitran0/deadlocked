use glam::Vec3;

use crate::cs2::CS2;

#[derive(Clone, Copy, Debug, Default)]
pub struct NetworkVelocityVector {
    x: f32,
    y: f32,
    z: f32,
}

impl NetworkVelocityVector {
    pub fn read(cs2: &CS2, address: usize) -> Self {
        let offsets = &cs2.offsets.network_velocity;
        Self {
            x: cs2.process.read(address + offsets.x),
            y: cs2.process.read(address + offsets.y),
            z: cs2.process.read(address + offsets.z),
        }
    }

    pub fn to_vec(self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }
}
