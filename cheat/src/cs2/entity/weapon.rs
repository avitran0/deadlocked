use shared::Weapon;

use crate::cs2::{CS2, entity::player::Player};

pub fn weapon_from_handle(handle: i32, cs2: &CS2) -> Option<Weapon> {
    if handle == 0 {
        return None;
    }
    let index = handle as usize & 0xFFF;
    let entity = Player::get_client_entity(cs2, index)?;
    Some(weapon_from_entity(entity, cs2))
}

pub fn weapon_from_entity(entity: usize, cs2: &CS2) -> Weapon {
    let weapon_index: u16 = cs2.process.read(
        entity
            + cs2.offsets.weapon.attribute_manager
            + cs2.offsets.weapon.item
            + cs2.offsets.econ_item_view.item_definition_index,
    );
    Weapon::from_index(weapon_index)
}

pub fn weapon_clip_ammo(entity: usize, cs2: &CS2) -> i32 {
    cs2.process.read(entity + cs2.offsets.weapon.clip_primary)
}

pub fn weapon_reserve_ammo(entity: usize, cs2: &CS2) -> i32 {
    cs2.process.read(entity + cs2.offsets.weapon.reserve_ammo)
}
