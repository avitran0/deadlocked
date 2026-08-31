use serde::{Deserialize, Serialize};
use strum::EnumIter;

use crate::WeaponClass;

#[derive(Default, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, EnumIter)]
pub enum Weapon {
    #[default]
    None,

    // pistols
    CZ75,
    DesertEagle,
    DualBerettas,
    FiveSeven,
    Glock,
    P2000,
    P250,
    Revolver,
    Tec9,
    Usp,

    // smg
    MAC10,
    MP5,
    MP7,
    MP9,
    P90,
    Bizon,
    UMP45,

    // shotguns
    Mag7,
    Nova,
    SawedOff,
    XM1014,

    // lmg
    M249,
    Negev,

    // assault rifles
    AK47,
    Aug,
    Famas,
    Galil,
    M4A1S,
    M4A4,
    SG553,

    // snipers
    Awp,
    G3SG1,
    SCAR20,
    SSG08,

    // knives
    KnifeCT,
    KnifeT,
    KnifeBayonet,
    KnifeBowie,
    KnifeButterfly,
    KnifeClassic,
    KnifeFalchion,
    KnifeFlip,
    KnifeGut,
    KnifeHuntsman,
    KnifeKarambit,
    KnifeKukri,
    KnifeM9Bayonet,
    KnifeNavaja,
    KnifeNomad,
    KnifeParacord,
    KnifeShadowDaggers,
    KnifeSkeleton,
    KnifeStiletto,
    KnifeSurvival,
    KnifeTalon,
    KnifeUrsus,
    KnifeGold,

    // grenades
    Flashbang,
    HE,
    Smoke,
    Molotov,
    Decoy,
    Incendiary,

    // misc
    Taser,
    C4,
    Healthshot,
}

impl Weapon {
    pub fn from_index(index: u16) -> Self {
        match index {
            1 => Self::DesertEagle,
            2 => Self::DualBerettas,
            3 => Self::FiveSeven,
            4 => Self::Glock,
            7 => Self::AK47,
            8 => Self::Aug,
            9 => Self::Awp,
            10 => Self::Famas,
            11 => Self::G3SG1,
            13 => Self::Galil,
            14 => Self::M249,
            16 => Self::M4A4,
            17 => Self::MAC10,
            19 => Self::P90,
            23 => Self::MP5,
            24 => Self::UMP45,
            25 => Self::XM1014,
            26 => Self::Bizon,
            27 => Self::Mag7,
            28 => Self::Negev,
            29 => Self::SawedOff,
            30 => Self::Tec9,
            31 => Self::Taser,
            32 => Self::P2000,
            33 => Self::MP7,
            34 => Self::MP9,
            35 => Self::Nova,
            36 => Self::P250,
            38 => Self::SCAR20,
            39 => Self::SG553,
            40 => Self::SSG08,
            41 => Self::KnifeGold,
            42 => Self::KnifeCT,
            43 => Self::Flashbang,
            44 => Self::HE,
            45 => Self::Smoke,
            46 => Self::Molotov,
            47 => Self::Decoy,
            48 => Self::Incendiary,
            49 => Self::C4,
            57 => Self::Healthshot,
            59 => Self::KnifeT,
            60 => Self::M4A1S,
            61 => Self::Usp,
            63 => Self::CZ75,
            64 => Self::Revolver,
            503 => Self::KnifeClassic,
            505 => Self::KnifeFlip,
            506 => Self::KnifeGut,
            507 => Self::KnifeKarambit,
            508 => Self::KnifeM9Bayonet,
            509 => Self::KnifeHuntsman,
            512 => Self::KnifeFalchion,
            514 => Self::KnifeBowie,
            515 => Self::KnifeButterfly,
            516 => Self::KnifeShadowDaggers,
            517 => Self::KnifeParacord,
            518 => Self::KnifeSurvival,
            519 => Self::KnifeUrsus,
            520 => Self::KnifeNavaja,
            521 => Self::KnifeNomad,
            522 => Self::KnifeStiletto,
            523 => Self::KnifeTalon,
            524 => Self::KnifeSkeleton,
            _ => Self::None,
        }
    }

    pub fn to_icon(&self) -> char {
        match self {
            Self::None => '?',

            // pistols
            Self::CZ75 => '\u{e02b}',
            Self::DesertEagle => '\u{e04a}',
            Self::DualBerettas => '\u{e056}',
            Self::FiveSeven => '\u{e03a}',
            Self::Glock => '\u{e023}',
            Self::P2000 => '\u{e05a}',
            Self::P250 => '\u{e04f}',
            Self::Revolver => '\u{e003}',
            Self::Tec9 => '\u{e00e}',
            Self::Usp => '\u{e037}',

            // smg
            Self::MAC10 => '\u{e049}',
            Self::MP5 => '\u{e039}',
            Self::MP7 => '\u{e047}',
            Self::MP9 => '\u{e040}',
            Self::P90 => '\u{e035}',
            Self::Bizon => '\u{e00c}',
            Self::UMP45 => '\u{e01f}',

            // shotguns
            Self::Mag7 => '\u{e032}',
            Self::Nova => '\u{e02d}',
            Self::SawedOff => '\u{e031}',
            Self::XM1014 => '\u{e01d}',

            // lmg
            Self::M249 => '\u{e055}',
            Self::Negev => '\u{e019}',

            // assault rifles
            Self::AK47 => '\u{e008}',
            Self::Aug => '\u{e060}',
            Self::Famas => '\u{e025}',
            Self::Galil => '\u{e000}',
            Self::M4A1S => '\u{e015}',
            Self::M4A4 => '\u{e042}',
            Self::SG553 => '\u{e00d}',

            // snipers
            Self::Awp => '\u{e007}',
            Self::G3SG1 => '\u{e04b}',
            Self::SCAR20 => '\u{e009}',
            Self::SSG08 => '\u{e03e}',

            // knives
            Self::KnifeCT => '\u{e050}',
            Self::KnifeT => '\u{e05b}',
            Self::KnifeBayonet => '\u{e014}',
            Self::KnifeBowie => '\u{e052}',
            Self::KnifeButterfly => '\u{e026}',
            Self::KnifeClassic => '\u{e04c}',
            Self::KnifeFalchion => '\u{e013}',
            Self::KnifeFlip => '\u{e01c}',
            Self::KnifeGut => '\u{e041}',
            Self::KnifeHuntsman => '\u{e010}',
            Self::KnifeKarambit => '\u{e048}',
            Self::KnifeKukri => '\u{e02f}',
            Self::KnifeM9Bayonet => '\u{e030}',
            Self::KnifeNavaja => '\u{e011}',
            Self::KnifeNomad => '\u{e058}',
            Self::KnifeParacord => '\u{e05d}',
            Self::KnifeShadowDaggers => '\u{e046}',
            Self::KnifeSkeleton => '\u{e005}',
            Self::KnifeStiletto => '\u{e054}',
            Self::KnifeSurvival => '\u{e002}',
            Self::KnifeTalon => '\u{e004}',
            Self::KnifeUrsus => '\u{e03c}',
            Self::KnifeGold => '\u{e044}',
            // grenades
            Self::Flashbang => '\u{e05e}',
            Self::HE => '\u{e00b}',
            Self::Smoke => '\u{e02e}',
            Self::Molotov => '\u{e024}',
            Self::Decoy => '\u{e028}',
            Self::Incendiary => '\u{e01a}',

            // misc
            Self::Taser => '\u{e021}',
            Self::C4 => '\u{e01e}',
            Self::Healthshot => '\u{e01b}',
        }
    }

    pub fn weapon_class(&self) -> WeaponClass {
        match self {
            Self::None => WeaponClass::Unknown,

            // pistols
            Self::CZ75 => WeaponClass::Pistol,
            Self::DesertEagle => WeaponClass::Pistol,
            Self::DualBerettas => WeaponClass::Pistol,
            Self::FiveSeven => WeaponClass::Pistol,
            Self::Glock => WeaponClass::Pistol,
            Self::P2000 => WeaponClass::Pistol,
            Self::P250 => WeaponClass::Pistol,
            Self::Revolver => WeaponClass::Pistol,
            Self::Tec9 => WeaponClass::Pistol,
            Self::Usp => WeaponClass::Pistol,

            // smg
            Self::MAC10 => WeaponClass::Smg,
            Self::MP5 => WeaponClass::Smg,
            Self::MP7 => WeaponClass::Smg,
            Self::MP9 => WeaponClass::Smg,
            Self::P90 => WeaponClass::Smg,
            Self::Bizon => WeaponClass::Smg,
            Self::UMP45 => WeaponClass::Smg,

            // shotguns
            Self::Mag7 => WeaponClass::Shotgun,
            Self::Nova => WeaponClass::Shotgun,
            Self::SawedOff => WeaponClass::Shotgun,
            Self::XM1014 => WeaponClass::Shotgun,

            // lmg
            Self::M249 => WeaponClass::Heavy,
            Self::Negev => WeaponClass::Heavy,

            // assault rifles
            Self::AK47 => WeaponClass::Rifle,
            Self::Aug => WeaponClass::Rifle,
            Self::Famas => WeaponClass::Rifle,
            Self::Galil => WeaponClass::Rifle,
            Self::M4A1S => WeaponClass::Rifle,
            Self::M4A4 => WeaponClass::Rifle,
            Self::SG553 => WeaponClass::Rifle,

            // snipers
            Self::Awp => WeaponClass::Sniper,
            Self::G3SG1 => WeaponClass::Sniper,
            Self::SCAR20 => WeaponClass::Sniper,
            Self::SSG08 => WeaponClass::Sniper,

            // knives
            Self::KnifeCT => WeaponClass::Knife,
            Self::KnifeT => WeaponClass::Knife,
            Self::KnifeBayonet => WeaponClass::Knife,
            Self::KnifeBowie => WeaponClass::Knife,
            Self::KnifeButterfly => WeaponClass::Knife,
            Self::KnifeClassic => WeaponClass::Knife,
            Self::KnifeFalchion => WeaponClass::Knife,
            Self::KnifeFlip => WeaponClass::Knife,
            Self::KnifeGut => WeaponClass::Knife,
            Self::KnifeHuntsman => WeaponClass::Knife,
            Self::KnifeKarambit => WeaponClass::Knife,
            Self::KnifeKukri => WeaponClass::Knife,
            Self::KnifeM9Bayonet => WeaponClass::Knife,
            Self::KnifeNavaja => WeaponClass::Knife,
            Self::KnifeNomad => WeaponClass::Knife,
            Self::KnifeParacord => WeaponClass::Knife,
            Self::KnifeShadowDaggers => WeaponClass::Knife,
            Self::KnifeSkeleton => WeaponClass::Knife,
            Self::KnifeStiletto => WeaponClass::Knife,
            Self::KnifeSurvival => WeaponClass::Knife,
            Self::KnifeTalon => WeaponClass::Knife,
            Self::KnifeUrsus => WeaponClass::Knife,
            Self::KnifeGold => WeaponClass::Knife,
            // grenades
            Self::Flashbang => WeaponClass::Grenade,
            Self::HE => WeaponClass::Grenade,
            Self::Smoke => WeaponClass::Grenade,
            Self::Molotov => WeaponClass::Grenade,
            Self::Decoy => WeaponClass::Grenade,
            Self::Incendiary => WeaponClass::Grenade,

            // misc
            Self::Taser => WeaponClass::Utility,
            Self::C4 => WeaponClass::Utility,
            Self::Healthshot => WeaponClass::Utility,
        }
    }
}

impl std::fmt::Display for Weapon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => "?",

            // pistols
            Self::CZ75 => "CZ75-Auto",
            Self::DesertEagle => "Desert Eagle",
            Self::DualBerettas => "Dual Berettas",
            Self::FiveSeven => "Five-SeveN",
            Self::Glock => "Glock-18",
            Self::P2000 => "P2000",
            Self::P250 => "P250",
            Self::Revolver => "R8 Revolver",
            Self::Tec9 => "Tec-9",
            Self::Usp => "USP-S",

            // smg
            Self::MAC10 => "MAC-10",
            Self::MP5 => "MP5-SD",
            Self::MP7 => "MP7",
            Self::MP9 => "MP9",
            Self::P90 => "P90",
            Self::Bizon => "PP-Bizon",
            Self::UMP45 => "UMP-45",

            // shotguns
            Self::Mag7 => "Mag-7",
            Self::Nova => "Nova",
            Self::SawedOff => "Sawed-Off",
            Self::XM1014 => "XM1014",

            // lmg
            Self::M249 => "M249",
            Self::Negev => "Negev",

            // assault rifles
            Self::AK47 => "AK-47",
            Self::Aug => "AUG",
            Self::Famas => "FAMAS",
            Self::Galil => "Galil AR",
            Self::M4A1S => "M4A1-S",
            Self::M4A4 => "M4A4",
            Self::SG553 => "SG 553",

            // snipers
            Self::Awp => "AWP",
            Self::G3SG1 => "G3SG1",
            Self::SCAR20 => "SCAR-20",
            Self::SSG08 => "SSG 08",

            // knives
            Self::KnifeCT => "Knife",
            Self::KnifeT => "Knife",
            Self::KnifeBayonet => "Bayonet",
            Self::KnifeBowie => "Bowie Knife",
            Self::KnifeButterfly => "Butterfly Knife",
            Self::KnifeClassic => "Classic Knife",
            Self::KnifeFalchion => "Falchion Knife",
            Self::KnifeFlip => "Flip Knife",
            Self::KnifeGut => "Gut Knife",
            Self::KnifeHuntsman => "Huntsman Knife",
            Self::KnifeKarambit => "Karambit",
            Self::KnifeKukri => "Kukri Knife",
            Self::KnifeM9Bayonet => "M9 Bayonet",
            Self::KnifeNavaja => "Navaja Knife",
            Self::KnifeNomad => "Nomad Knife",
            Self::KnifeParacord => "Paracord Knife",
            Self::KnifeShadowDaggers => "Shadow Daggers",
            Self::KnifeSkeleton => "Skeleton Knife",
            Self::KnifeStiletto => "Stiletto Knife",
            Self::KnifeSurvival => "Survival Knife",
            Self::KnifeTalon => "Talon Knife",
            Self::KnifeUrsus => "Ursus Knife",
            Self::KnifeGold => "Gold Knife",
            // grenades
            Self::Flashbang => "Flashbang",
            Self::HE => "HE Grenade",
            Self::Smoke => "Smoke Grenade",
            Self::Molotov => "Molotov",
            Self::Decoy => "Decoy Grenade",
            Self::Incendiary => "Incendiary Grenade",

            // misc
            Self::Taser => "Zeus x27",
            Self::C4 => "C4",
            Self::Healthshot => "Healthshot",
        }
        .fmt(f)
    }
}
