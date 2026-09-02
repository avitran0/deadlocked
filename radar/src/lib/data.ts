export interface Data {
    in_game: boolean;
    is_ffa: boolean;
    weapon: string;
    players: PlayerData[];
    friendlies: PlayerData[];
    local_player: PlayerData;
    entities: EntityInfo[];
    bomb: BombData;
    map_name: string;
}

export interface PlayerData {
    health: number;
    max_health: number;
    armor: number;
    position: Vec3;
    name: string;
    weapon: string;
    ammo: Ammo;
    has_defuser: boolean;
    has_helmet: boolean;
    has_bomb: boolean;
    visible: boolean;
    color: Color;
    rotation: number;
}

export interface BombData {
    planted: boolean;
    timer: number;
    being_defused: boolean;
    position: Vec3;
    defuse_remain_time: number;
}

export type EntityInfo =
    | { type: "Weapon"; weapon: string; position: Vec3; ammo: Ammo }
    | { type: "Inferno"; position: Vec3; hull: Vec3[] }
    | { type: "Molotov"; position: Vec3; is_incendiary: boolean }
    | { type: "Smoke"; position: Vec3; name: string }
    | { type: "Flashbang"; position: Vec3; name: string }
    | { type: "HeGrenade"; position: Vec3; name: string }
    | { type: "Decoy"; position: Vec3; name: string }
    | { type: "Chicken"; position: Vec3; visible: boolean };

export type Vec2 = [x: number, y: number];
export type Vec3 = [x: number, y: number, z: number];
export type Ammo = [clip: number, reserve: number];

// todo: cross-check this
export enum Color {
    Yellow = 0,
    Purple = 1,
    Green = 2,
    Blue = 3,
    Orange = 4,
}
