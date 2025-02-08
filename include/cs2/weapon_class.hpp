#include <string>
#include <unordered_map>

enum class WeaponClass {
    Unknown,
    Knife,
    Pistol,
    Smg,
    Heavy,   // shotguns and lmgs
    Rifle,   // all rifles except snipers
    Sniper,  // these require different handling in aimbot
    Grenade,
    Utility  // taser
};

WeaponClass WeaponClassFromString(const std::string &name);
