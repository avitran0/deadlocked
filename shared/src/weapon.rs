use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter};

#[derive(
    Debug, Default, Clone, PartialEq, Eq, Hash, EnumIter, AsRefStr, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum Weapon {
    #[default]
    Unknown,

    Knife,

    // Pistols
    Cz75A,
    Deagle,
    DualBerettas,
    FiveSeven,
    Glock,
    P2000,
    P250,
    Revolver,
    Tec9,
    Usp,

    // SMGs
    Bizon,
    Mac10,
    Mp5Sd,
    Mp7,
    Mp9,
    P90,
    Ump45,

    // LMGs
    M249,
    Negev,

    // Shotguns
    Mag7,
    Nova,
    Sawedoff,
    Xm1014,

    // Rifles
    Ak47,
    Aug,
    Famas,
    Galilar,
    M4A4,
    M4A1,
    Sg556,

    // Snipers
    Awp,
    G3SG1,
    Scar20,
    Ssg08,

    // Utility
    Taser,

    // Grenades
    Flashbang,
    HeGrenade,
    Smoke,
    Molotov,
    Decoy,
    Incendiary,

    // Bomb
    C4,
}

impl std::fmt::Display for Weapon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Weapon::Unknown => "Unknown",
            Weapon::Knife => "Knife",
            Weapon::Cz75A => "CZ75-Auto",
            Weapon::Deagle => "Desert Eagle",
            Weapon::DualBerettas => "Dual Berettas",
            Weapon::FiveSeven => "Five-SeveN",
            Weapon::Glock => "Glock-18",
            Weapon::P2000 => "P2000",
            Weapon::P250 => "P250",
            Weapon::Revolver => "R8 Revolver",
            Weapon::Tec9 => "Tec-9",
            Weapon::Usp => "USP-S",
            Weapon::Bizon => "PP-Bizon",
            Weapon::Mac10 => "MAC-10",
            Weapon::Mp5Sd => "MP5-SD",
            Weapon::Mp7 => "MP7",
            Weapon::Mp9 => "MP9",
            Weapon::P90 => "P90",
            Weapon::Ump45 => "UMP-45",
            Weapon::M249 => "M249",
            Weapon::Negev => "Negev",
            Weapon::Mag7 => "MAG-7",
            Weapon::Nova => "Nova",
            Weapon::Sawedoff => "Sawed-Off",
            Weapon::Xm1014 => "XM1014",
            Weapon::Ak47 => "AK-47",
            Weapon::Aug => "AUG",
            Weapon::Famas => "FAMAS",
            Weapon::Galilar => "Galil AR",
            Weapon::M4A4 => "M4A4",
            Weapon::M4A1 => "M4A1-S",
            Weapon::Sg556 => "SG 553",
            Weapon::Awp => "AWP",
            Weapon::G3SG1 => "G3SG1",
            Weapon::Scar20 => "SCAR-20",
            Weapon::Ssg08 => "SSG 08",
            Weapon::Taser => "Zeus x27",
            Weapon::Flashbang => "Flashbang",
            Weapon::HeGrenade => "HE Grenade",
            Weapon::Smoke => "Smoke Grenade",
            Weapon::Molotov => "Molotov Cocktail",
            Weapon::Decoy => "Decoy Grenade",
            Weapon::Incendiary => "Incendiary Grenade",
            Weapon::C4 => "C4 Explosive",
        };
        write!(f, "{}", s)
    }
}
