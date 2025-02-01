use std::collections::HashMap;

use glam::{Vec2, Vec3};

use super::{bones::Bones, weapon_class::WeaponClass};

pub struct Player {
    controller: u64,
    pawn: u64,

    health: i32,
    team: u8,
    life_state: u8,
    position: Vec3,
    visible: bool,
    bones: HashMap<Bones, Vec3>,
}

pub struct LocalPlayer {
    controller: u64,
    pawn: u64,

    health: i32,
    team: u8,
    weapon_class: WeaponClass,
    position: Vec3,
    eye_position: Vec3,
    shots_fired: i32,
    sensitivity: f32,
    view_angles: Vec2,
    aim_punch: Vec2,
}
