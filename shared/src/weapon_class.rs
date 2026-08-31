#[derive(Default, PartialEq)]
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