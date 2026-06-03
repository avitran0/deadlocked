use std::{collections::HashMap, fs::read_to_string};

use glam::{Vec2, Vec3};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{config::BASE_PATH, constants::GRENADE_FILE_NAME, cs2::entity::weapon::Weapon};

pub type GrenadeList = HashMap<String, Vec<Grenade>>;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Grenade {
    pub id: Uuid,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub position: Vec3,
    pub view_angles: Vec2,
    pub weapon: Weapon,
    #[serde(default)]
    pub modifiers: GrenadeModifiers,
    #[serde(default)]
    pub automation: GrenadeAutomation,
}

impl Grenade {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct GrenadeModifiers {
    pub jump: bool,
    pub duck: bool,
    pub run: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThrowButtons {
    Left,
    Right,
    Both,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(default)]
pub struct GrenadeAutomation {
    pub movement_flags: u32,
    pub always_run: bool,
    pub run_ticks: u32,
    pub jump_delay_ticks: u32,
    pub throw_strength: f32,
}

impl Default for GrenadeAutomation {
    fn default() -> Self {
        Self {
            movement_flags: 0,
            always_run: false,
            run_ticks: 0,
            jump_delay_ticks: 0,
            throw_strength: 1.0,
        }
    }
}

impl GrenadeAutomation {
    pub fn throw_buttons(&self, description: &str) -> ThrowButtons {
        let description = description.to_ascii_lowercase();
        let compact = description
            .chars()
            .filter(|ch| ch.is_ascii_alphanumeric())
            .collect::<String>();

        if description.contains("lrmb")
            || description.contains("lmb+rmb")
            || description.contains("rmb+lmb")
            || description.contains("lmb + rmb")
            || description.contains("rmb + lmb")
            || description.contains("left and right")
            || description.contains("right and left")
            || description.contains("both")
            || compact.contains("lmbrmb")
            || compact.contains("rmblmb")
            || compact.contains("leftright")
            || compact.contains("rightleft")
            || compact.contains("mouse1mouse2")
            || compact.contains("mouse2mouse1")
            || self.throw_strength < 0.75
        {
            return ThrowButtons::Both;
        }
        if description.contains("rmb") {
            return ThrowButtons::Right;
        }
        ThrowButtons::Left
    }
}

pub fn read_grenades() -> GrenadeList {
    let path = BASE_PATH.join(GRENADE_FILE_NAME);
    if !path.exists() {
        utils::info!("no grenade list found");
        return GrenadeList::default();
    }

    let grenade_list_file = read_to_string(path).unwrap();
    let grenade_list = serde_json::from_str(&grenade_list_file);
    if grenade_list.is_err() {
        utils::warn!("grenade list file invalid");
    }
    grenade_list.unwrap_or_default()
}

pub fn write_grenades(grenades: &GrenadeList) {
    let out = serde_json::to_string(grenades).unwrap();
    let path = BASE_PATH.join(GRENADE_FILE_NAME);
    std::fs::write(path, out).unwrap();
}

#[cfg(test)]
mod tests {
    use super::{Grenade, GrenadeAutomation, ThrowButtons};

    #[test]
    fn old_grenade_json_defaults_automation() {
        let json = r#"{
            "id": "1b454cdb-c300-48f3-a3b8-d68940d324fc",
            "name": "Top Six",
            "description": "Jump+Throw",
            "position": [-1340.0552, 1463.9781, -169.9754],
            "view_angles": [-52.909962, -31.269911],
            "weapon": "smoke",
            "modifiers": {"jump": true, "duck": false, "run": false}
        }"#;

        let grenade = serde_json::from_str::<Grenade>(json).unwrap();

        assert_eq!(grenade.automation, GrenadeAutomation::default());
    }

    #[test]
    fn imported_automation_fields_round_trip() {
        let json = r#"{
            "id": "1b454cdb-c300-48f3-a3b8-d68940d324fc",
            "name": "B Default",
            "description": "W+Jump+Throw LRMB",
            "position": [1.0, 2.0, 3.0],
            "view_angles": [4.0, 5.0],
            "weapon": "molotov",
            "modifiers": {"jump": true, "duck": false, "run": true},
            "automation": {
                "movement_flags": 518,
                "always_run": true,
                "run_ticks": 1,
                "jump_delay_ticks": 1,
                "throw_strength": 0.5
            }
        }"#;

        let grenade = serde_json::from_str::<Grenade>(json).unwrap();

        assert_eq!(grenade.automation.movement_flags, 518);
        assert!(grenade.automation.always_run);
        assert_eq!(grenade.automation.run_ticks, 1);
        assert_eq!(grenade.automation.jump_delay_ticks, 1);
        assert_eq!(
            grenade.automation.throw_buttons(&grenade.description),
            ThrowButtons::Both
        );
    }

    #[test]
    fn throw_button_inference_uses_description_and_strength() {
        let left = GrenadeAutomation {
            throw_strength: 1.0,
            ..Default::default()
        };
        let both = GrenadeAutomation {
            throw_strength: 0.5,
            ..Default::default()
        };

        assert_eq!(left.throw_buttons("Jump+Throw"), ThrowButtons::Left);
        assert_eq!(left.throw_buttons("Throw RMB"), ThrowButtons::Right);
        assert_eq!(both.throw_buttons("Throw LRMB"), ThrowButtons::Both);
        assert_eq!(left.throw_buttons("Throw LMB+RMB"), ThrowButtons::Both);
        assert_eq!(
            left.throw_buttons("Throw left and right mouse button"),
            ThrowButtons::Both
        );
    }
}
