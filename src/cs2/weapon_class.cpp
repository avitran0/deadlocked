#include "cs2/weapon_class.hpp"

#include <unordered_map>

WeaponClass WeaponClassFromString(const std::string &name) {
    static const std::unordered_map<std::string, WeaponClass> weaponMap = {
        // Knives
        {"bayonet", WeaponClass::Knife},
        {"knife", WeaponClass::Knife},
        {"knife_bowie", WeaponClass::Knife},
        {"knife_butterfly", WeaponClass::Knife},
        {"knife_canis", WeaponClass::Knife},
        {"knife_cord", WeaponClass::Knife},
        {"knife_css", WeaponClass::Knife},
        {"knife_falchion", WeaponClass::Knife},
        {"knife_flip", WeaponClass::Knife},
        {"knife_gut", WeaponClass::Knife},
        {"knife_gypsy_jackknife", WeaponClass::Knife},
        {"knife_karambit", WeaponClass::Knife},
        {"knife_kukri", WeaponClass::Knife},
        {"knife_m9_bayonet", WeaponClass::Knife},
        {"knife_outdoor", WeaponClass::Knife},
        {"knife_push", WeaponClass::Knife},
        {"knife_skeleton", WeaponClass::Knife},
        {"knife_stiletto", WeaponClass::Knife},
        {"knife_survival_bowie", WeaponClass::Knife},
        {"knife_t", WeaponClass::Knife},
        {"knife_tactical", WeaponClass::Knife},
        {"knife_twinblade", WeaponClass::Knife},
        {"knife_ursus", WeaponClass::Knife},
        {"knife_widowmaker", WeaponClass::Knife},

        // Pistols
        {"cz75a", WeaponClass::Pistol},
        {"deagle", WeaponClass::Pistol},
        {"elite", WeaponClass::Pistol},
        {"fiveseven", WeaponClass::Pistol},
        {"glock", WeaponClass::Pistol},
        {"hkp2000", WeaponClass::Pistol},
        {"p250", WeaponClass::Pistol},
        {"revolver", WeaponClass::Pistol},
        {"tec9", WeaponClass::Pistol},
        {"usp_silencer", WeaponClass::Pistol},
        {"usp_silencer_off", WeaponClass::Pistol},

        // SMGs
        {"bizon", WeaponClass::Smg},
        {"mac10", WeaponClass::Smg},
        {"mp5sd", WeaponClass::Smg},
        {"mp7", WeaponClass::Smg},
        {"mp9", WeaponClass::Smg},
        {"p90", WeaponClass::Smg},
        {"ump45", WeaponClass::Smg},

        // Heavy
        {"m249", WeaponClass::Heavy},
        {"negev", WeaponClass::Heavy},

        // Shotguns
        {"mag7", WeaponClass::Shotgun},
        {"nova", WeaponClass::Shotgun},
        {"sawedoff", WeaponClass::Shotgun},
        {"xm1014", WeaponClass::Shotgun},

        // Rifles
        {"ak47", WeaponClass::Rifle},
        {"aug", WeaponClass::Rifle},
        {"famas", WeaponClass::Rifle},
        {"galilar", WeaponClass::Rifle},
        {"m4a1_silencer", WeaponClass::Rifle},
        {"m4a1_silencer_off", WeaponClass::Rifle},
        {"m4a1", WeaponClass::Rifle},
        {"sg556", WeaponClass::Rifle},

        // Snipers
        {"awp", WeaponClass::Sniper},
        {"g3sg1", WeaponClass::Sniper},
        {"scar20", WeaponClass::Sniper},
        {"ssg08", WeaponClass::Sniper},

        // Grenades
        {"decoy", WeaponClass::Grenade},
        {"firebomb", WeaponClass::Grenade},
        {"flashbang", WeaponClass::Grenade},
        {"frag_grenade", WeaponClass::Grenade},
        {"hegrenade", WeaponClass::Grenade},
        {"incgrenade", WeaponClass::Grenade},
        {"molotov", WeaponClass::Grenade},
        {"smokegrenade", WeaponClass::Grenade},

        // Utility
        {"taser", WeaponClass::Utility},
    };

    const auto it = weaponMap.find(name);
    return it != weaponMap.end() ? it->second : WeaponClass::Unknown;
}
