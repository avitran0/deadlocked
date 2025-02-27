export enum WeaponCategory {
    Primary,
    Secondary,
    Knife,
    Grenade,
    Bomb,
    Utility,
}

export const WeaponCategories: Record<string, WeaponCategory> = {
    // Primary weapons
    ak47: WeaponCategory.Primary,
    aug: WeaponCategory.Primary,
    awp: WeaponCategory.Primary,
    bizon: WeaponCategory.Primary,
    famas: WeaponCategory.Primary,
    g3sg1: WeaponCategory.Primary,
    galilar: WeaponCategory.Primary,
    m249: WeaponCategory.Primary,
    m4a1_silencer: WeaponCategory.Primary,
    m4a1_silencer_off: WeaponCategory.Primary,
    m4a1: WeaponCategory.Primary,
    mac10: WeaponCategory.Primary,
    mag7: WeaponCategory.Primary,
    mp5sd: WeaponCategory.Primary,
    mp7: WeaponCategory.Primary,
    mp9: WeaponCategory.Primary,
    negev: WeaponCategory.Primary,
    nova: WeaponCategory.Primary,
    p90: WeaponCategory.Primary,
    sawedoff: WeaponCategory.Primary,
    scar20: WeaponCategory.Primary,
    sg556: WeaponCategory.Primary,
    ssg08: WeaponCategory.Primary,
    ump45: WeaponCategory.Primary,
    xm1014: WeaponCategory.Primary,

    // Secondary weapons
    cz75a: WeaponCategory.Secondary,
    deagle: WeaponCategory.Secondary,
    elite: WeaponCategory.Secondary,
    fiveseven: WeaponCategory.Secondary,
    glock: WeaponCategory.Secondary,
    hkp2000: WeaponCategory.Secondary,
    p2000: WeaponCategory.Secondary,
    p250: WeaponCategory.Secondary,
    revolver: WeaponCategory.Secondary,
    tec9: WeaponCategory.Secondary,
    usp_silencer: WeaponCategory.Secondary,
    usp_silencer_off: WeaponCategory.Secondary,

    // Knives
    bayonet: WeaponCategory.Knife,
    knife: WeaponCategory.Knife,
    knife_bowie: WeaponCategory.Knife,
    knife_butterfly: WeaponCategory.Knife,
    knife_canis: WeaponCategory.Knife,
    knife_cord: WeaponCategory.Knife,
    knife_css: WeaponCategory.Knife,
    knife_falchion: WeaponCategory.Knife,
    knife_flip: WeaponCategory.Knife,
    knife_gut: WeaponCategory.Knife,
    knife_gypsy_jackknife: WeaponCategory.Knife,
    knife_karambit: WeaponCategory.Knife,
    knife_kukri: WeaponCategory.Knife,
    knife_m9_bayonet: WeaponCategory.Knife,
    knife_outdoor: WeaponCategory.Knife,
    knife_push: WeaponCategory.Knife,
    knife_skeleton: WeaponCategory.Knife,
    knife_stiletto: WeaponCategory.Knife,
    knife_survival_bowie: WeaponCategory.Knife,
    knife_t: WeaponCategory.Knife,
    knife_tactical: WeaponCategory.Knife,
    knife_twinblade: WeaponCategory.Knife,
    knife_ursus: WeaponCategory.Knife,
    knife_widowmaker: WeaponCategory.Knife,

    // Grenades
    decoy: WeaponCategory.Grenade,
    firebomb: WeaponCategory.Grenade,
    flashbang: WeaponCategory.Grenade,
    frag_grenade: WeaponCategory.Grenade,
    hegrenade: WeaponCategory.Grenade,
    incgrenade: WeaponCategory.Grenade,
    molotov: WeaponCategory.Grenade,
    smokegrenade: WeaponCategory.Grenade,

    // Bomb
    c4: WeaponCategory.Bomb,

    // Utility
    taser: WeaponCategory.Utility,
};
