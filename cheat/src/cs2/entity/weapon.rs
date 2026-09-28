use shared::Weapon;

use crate::cs2::{CS2, entity::player::Player};

#[derive(Clone, Copy)]
pub struct WeaponEntity {
    entity: usize,
}

impl WeaponEntity {
    pub fn new(entity: usize) -> Self {
        Self { entity }
    }

    pub fn vdata(&self, cs2: &CS2) -> Option<WeaponVData> {
        let address: usize = cs2
            .process
            .read(self.entity + cs2.offsets.entity.subclass_vdata);
        if address == 0 {
            return None;
        }

        let offsets = &cs2.offsets.weapon_vdata;
        let data = WeaponVData {
            damage: cs2.process.read(address + offsets.damage),
            headshot_multiplier: cs2.process.read(address + offsets.headshot_multiplier),
            armor_ratio: cs2.process.read(address + offsets.armor_ratio),
            penetration: cs2.process.read(address + offsets.penetration),
            range: cs2.process.read(address + offsets.range),
            range_modifier: cs2.process.read(address + offsets.range_modifier),
        };
        data.is_valid().then_some(data)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct WeaponVData {
    pub damage: i32,
    pub headshot_multiplier: f32,
    pub armor_ratio: f32,
    pub penetration: f32,
    pub range: f32,
    pub range_modifier: f32,
}

impl WeaponVData {
    fn is_valid(&self) -> bool {
        self.damage > 0
            && self.headshot_multiplier.is_finite()
            && self.headshot_multiplier >= 0.0
            && self.armor_ratio.is_finite()
            && self.armor_ratio >= 0.0
            && self.penetration.is_finite()
            && self.penetration > 0.0
            && self.range.is_finite()
            && self.range > 0.0
            && self.range_modifier.is_finite()
            && self.range_modifier > 0.0
    }
}

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
