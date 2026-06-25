#[derive(Debug, PartialEq, Default)]
pub enum WeaponClass {
    #[default]
    Unknown,
    Knife,
    Pistol,
    Smg,
    Heavy, // negev and m249
    Shotgun,
    Rifle,  // all rifles except snipers
    Sniper, // these require different handling in aimbot
    Grenade,
    Utility, // taser
}

impl WeaponClass {
    pub fn from_string(name: &str) -> Self {
        match name {
            // Knives
            "bayonet"
            | "knife"
            | "knife_bowie"
            | "knife_butterfly"
            | "knife_canis"
            | "knife_cord"
            | "knife_css"
            | "knife_falchion"
            | "knife_flip"
            | "knife_gut"
            | "knife_gypsy_jackknife"
            | "knife_karambit"
            | "knife_kukri"
            | "knife_m9_bayonet"
            | "knife_outdoor"
            | "knife_push"
            | "knife_skeleton"
            | "knife_stiletto"
            | "knife_survival_bowie"
            | "knife_t"
            | "knife_tactical"
            | "knife_twinblade"
            | "knife_ursus"
            | "knife_widowmaker" => WeaponClass::Knife,

            // Pistols
            "cz75a" | "deagle" | "elite" | "fiveseven" | "glock" | "hkp2000" | "p250"
            | "revolver" | "tec9" | "usp_silencer" | "usp_silencer_off" => WeaponClass::Pistol,

            // SMGs
            "bizon" | "mac10" | "mp5sd" | "mp7" | "mp9" | "p90" | "ump45" => WeaponClass::Smg,

            // LMGs
            "m249" | "negev" => WeaponClass::Heavy,

            // Shotguns
            "mag7" | "nova" | "sawedoff" | "xm1014" => WeaponClass::Shotgun,

            // Rifles
            "ak47" | "aug" | "famas" | "galilar" | "m4a1_silencer" | "m4a1_silencer_off"
            | "m4a1" | "sg556" => WeaponClass::Rifle,

            // Snipers
            "awp" | "g3sg1" | "scar20" | "ssg08" => WeaponClass::Sniper,

            // Grenades
            "flashbang" | "hegrenade" | "smokegrenade" | "molotov" | "decoy" | "incgrenade" => {
                WeaponClass::Grenade
            }

            // Utility
            "taser" => WeaponClass::Utility,

            // Default case: unknown weapon
            _ => WeaponClass::Unknown,
        }
    }
}
