use glam::Vec3;
use serde::{Deserialize, Serialize};

use crate::weapon::Weapon;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntityInfo {
    Weapon {
        weapon: Weapon,
        position: Vec3,
        ammo: (i32, i32),
    },
    Inferno(InfernoInfo),
    Molotov(MolotovInfo),
    Smoke(GrenadeInfo),
    Flashbang(GrenadeInfo),
    HeGrenade(GrenadeInfo),
    Decoy(GrenadeInfo),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrenadeInfo {
    pub entity: u64,
    pub position: Vec3,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfernoInfo {
    pub entity: u64,
    pub position: Vec3,
    pub hull: Vec<Vec3>,
}

impl InfernoInfo {
    pub fn grenade(&self) -> GrenadeInfo {
        GrenadeInfo {
            entity: self.entity,
            position: self.position,
            name: "Inferno".to_owned(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MolotovInfo {
    pub entity: u64,
    pub position: Vec3,
    pub is_incendiary: bool,
}

impl MolotovInfo {
    pub fn grenade(&self) -> GrenadeInfo {
        GrenadeInfo {
            entity: self.entity,
            position: self.position,
            name: if self.is_incendiary {
                "Incendiary"
            } else {
                "Molotov"
            }
            .to_owned(),
        }
    }
}
