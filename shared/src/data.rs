use std::collections::HashMap;

use glam::{Mat4, Vec2, Vec3};
use serde::{Deserialize, Serialize};

use crate::{bones::Bones, entity::EntityInfo, team::Team, weapon::Weapon};

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SoundType {
    Footstep,
    Gunshot,
    Weapon,
}

// lots of stuff is skipped, that is only for tcp postcard serialization
#[derive(Default, Serialize, Deserialize)]
pub struct Data {
    pub in_game: bool,
    pub is_ffa: bool,
    pub weapon: Weapon,
    pub players: Vec<PlayerData>,
    pub friendlies: Vec<PlayerData>,
    pub spectators: Vec<String>,
    pub local_player: PlayerData,
    pub entities: Vec<EntityInfo>,
    pub bomb: BombData,
    pub map_name: String,
    #[serde(skip)]
    pub view_matrix: Mat4,
    #[serde(skip)]
    pub view_angles: Vec2,
    #[serde(skip)]
    pub window_position: Vec2,
    #[serde(skip)]
    pub window_size: Vec2,
    #[serde(skip)]
    pub aimbot_active: bool,
    #[serde(skip)]
    pub triggerbot_active: bool,
    #[serde(skip)]
    pub esp_active: bool,
}

#[derive(Default, Serialize, Deserialize)]
pub struct PlayerData {
    pub steam_id: u64,
    pub money: i32,
    pub team: Team,
    pub health: i32,
    pub max_health: i32,
    pub armor: i32,
    pub position: Vec3,
    #[serde(skip)]
    pub head: Vec3,
    pub name: String,
    pub weapon: Weapon,
    pub ammo: (i32, i32),
    #[serde(skip)]
    pub bones: HashMap<Bones, Vec3>,
    pub has_defuser: bool,
    pub has_helmet: bool,
    pub has_bomb: bool,
    #[serde(skip)]
    pub visible: bool,
    pub color: i32,
    pub rotation: f32,
    #[serde(skip)]
    pub sound: Option<SoundType>,
    #[serde(skip)]
    pub collision_mins: Vec3,
    #[serde(skip)]
    pub collision_maxs: Vec3,
    #[serde(skip)]
    pub collision_transform: Mat4,
}

#[derive(Default, Serialize, Deserialize)]
pub struct BombData {
    pub planted: bool,
    pub timer: f32,
    pub being_defused: bool,
    pub position: Vec3,
    pub defuse_remain_time: f32,
}
