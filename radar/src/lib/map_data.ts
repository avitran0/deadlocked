import type { Vec2, Vec3 } from "./data";

export const MAP_DATA: Record<string, MapData> = {
    ar_baggage: {
        posX: -1316.0,
        posY: 1288.0,
        scale: 2.539062,
        rotate: 1.0,
        zoom: 1.3,
        lowerTransition: -5.0,
    },
    ar_shoots: {
        posX: -1368.0,
        posY: 1952.0,
        scale: 2.6875,
    },
    ar_shoots_night: {
        posX: -1368.0,
        posY: 1952.0,
        scale: 2.6875,
    },
    cs_italy: {
        posX: -2647.0,
        posY: 2592.0,
        scale: 4.6,
        rotate: 1.0,
        zoom: 1.5,
    },
    cs_office: {
        posX: -1838.0,
        posY: 1858.0,
        scale: 4.1,
    },
    de_ancient: {
        posX: -2953.0,
        posY: 2164.0,
        scale: 5.0,
        rotate: 0.0,
        zoom: 0.0,
    },
    de_ancient_night: {
        posX: -2953.0,
        posY: 2164.0,
        scale: 5.0,
        rotate: 0.0,
        zoom: 0.0,
    },
    de_ancient_v1: {
        posX: -2953.0,
        posY: 2164.0,
        scale: 5.0,
        rotate: 0.0,
        zoom: 0.0,
    },
    de_ancient_v2: {
        posX: -2953.0,
        posY: 2164.0,
        scale: 5.0,
        rotate: 0.0,
        zoom: 0.0,
    },
    de_anubis: {
        posX: -2796.0,
        posY: 3328.0,
        scale: 5.22,
    },
    de_cache: {
        posX: -2000.0,
        posY: 3250.0,
        scale: 5.5,
    },
    de_dust: {
        posX: -2850.0,
        posY: 4073.0,
        scale: 6.0,
        rotate: 1.0,
        zoom: 1.3,
    },
    de_dust2: {
        posX: -2476.0,
        posY: 3239.0,
        scale: 4.4,
        rotate: 1.0,
        zoom: 1.1,
    },
    de_inferno: {
        posX: -2087.0,
        posY: 3870.0,
        scale: 4.9,
    },
    de_inferno_s2: {
        posX: -2087.0,
        posY: 3870.0,
        scale: 4.9,
    },
    de_mirage: {
        posX: -3230.0,
        posY: 1713.0,
        scale: 5.0,
        rotate: 0.0,
        zoom: 0.0,
    },
    de_nuke: {
        posX: -3453.0,
        posY: 2887.0,
        scale: 7.0,
        lowerTransition: -495.0,
    },
    de_overpass: {
        posX: -4831.0,
        posY: 1781.0,
        scale: 5.2,
        rotate: 0.0,
        zoom: 0.0,
    },
    de_overpass_2v2: {
        posX: -4831.0,
        posY: 1781.0,
        scale: 5.2,
        rotate: 0.0,
        zoom: 0.0,
    },
    de_train: {
        posX: -2308.0,
        posY: 2078.0,
        scale: 4.082077,
        lowerTransition: -50.0,
    },
    de_vertigo: {
        posX: -3168.0,
        posY: 1762.0,
        scale: 4.0,
        lowerTransition: 11700.0,
    },
    cs_shelter: {
        posX: -3448.7712,
        posY: 3805.3228,
        scale: 3.311857,
    },
    de_boulder: {
        posX: -3273.4917,
        posY: 2930.2207,
        scale: 2.8491137,
        lowerTransition: 62.757324,
    },
    de_debris: {
        posX: -3015.2393,
        posY: 2919.2393,
        scale: 1.9445695,
    },
    de_eldorado: {
        posX: -3548.6814,
        posY: 2571.1816,
        scale: 2.0656068,
    },
    de_fachwerk: {
        posX: -2311.0767,
        posY: 2874.6204,
        scale: 2.5537856,
    },
    de_poseidon: {
        posX: -1046.3943,
        posY: 1166.3942,
        scale: 3.0124886,
    },
};

export interface MapData {
    posX: number;
    posY: number;
    scale: number;
    rotate?: number;
    zoom?: number;
    lowerTransition?: number;
}

export function worldToRadar(world: Vec3, map: MapData): Vec2 {
    return [
        ((world[0] - map.posX) / map.scale / 1024) * 100,
        ((map.posY - world[1]) / map.scale / 1024) * 100,
    ];
}
