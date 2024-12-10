#[derive(Debug, PartialEq)]
pub enum WeaponClass {
    Unknown,
    Knife,
    Pistol,
    Smg,
    Heavy,  // shotguns and lmgs
    Rifle,  // all rifles except snipers
    Sniper, // these require different handling in aimbot
    Grenade,
    Utility, // taser
}

impl WeaponClass {
    pub fn from_string(name: &str) -> Self {
        match name {
            // Knives
            "weapon_bayonet"
            | "weapon_knife"
            | "weapon_knife_bowie"
            | "weapon_knife_butterfly"
            | "weapon_knife_canis"
            | "weapon_knife_cord"
            | "weapon_knife_css"
            | "weapon_knife_falchion"
            | "weapon_knife_flip"
            | "weapon_knife_gut"
            | "weapon_knife_gypsy_jackknife"
            | "weapon_knife_karambit"
            | "weapon_knife_kukri"
            | "weapon_knife_m9_bayonet"
            | "weapon_knife_outdoor"
            | "weapon_knife_push"
            | "weapon_knife_skeleton"
            | "weapon_knife_stiletto"
            | "weapon_knife_survival_bowie"
            | "weapon_knife_t"
            | "weapon_knife_tactical"
            | "weapon_knife_twinblade"
            | "weapon_knife_ursus"
            | "weapon_knife_widowmaker" => WeaponClass::Knife,

            // Pistols
            "weapon_cz75a"
            | "weapon_deagle"
            | "weapon_elite"
            | "weapon_fiveseven"
            | "weapon_glock"
            | "weapon_hkp2000"
            | "weapon_p2000"
            | "weapon_p250"
            | "weapon_revolver"
            | "weapon_tec9"
            | "weapon_usp_silencer"
            | "weapon_usp_silencer_off" => WeaponClass::Pistol,

            // SMGs
            "weapon_bizon" | "weapon_mac10" | "weapon_mp5sd" | "weapon_mp7" | "weapon_mp9"
            | "weapon_p90" | "weapon_ump45" => WeaponClass::Smg,

            // Heavy weapons (Shotguns & LMGs)
            "weapon_m249" | "weapon_negev" | "weapon_mag7" | "weapon_nova" | "weapon_sawedoff"
            | "weapon_xm1014" => WeaponClass::Heavy,

            // Rifles
            "weapon_ak47"
            | "weapon_aug"
            | "weapon_famas"
            | "weapon_galilar"
            | "weapon_m4a1_silencer"
            | "weapon_m4a1_silencer_off"
            | "weapon_m4a1"
            | "weapon_sg556" => WeaponClass::Rifle,

            // Snipers
            "weapon_awp" | "weapon_g3sg1" | "weapon_scar20" | "weapon_ssg08" => WeaponClass::Sniper,

            // Grenades
            "weapon_decoy"
            | "weapon_firebomb"
            | "weapon_flashbang"
            | "weapon_frag_grenade"
            | "weapon_hegrenade"
            | "weapon_incgrenade"
            | "weapon_molotov"
            | "weapon_smokegrenade" => WeaponClass::Grenade,

            // Utility
            "weapon_taser" => WeaponClass::Utility,

            // Default case: unknown weapon
            _ => WeaponClass::Unknown,
        }
    }
}
