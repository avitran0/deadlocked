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

export type Team = "Unassigned" | "Spectator" | "T" | "CT";

export interface PlayerData {
    team: Team;
    steam_id: number;
    money: number;
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
    | { Weapon: WeaponInfo }
    | { Inferno: InfernoInfo }
    | { Molotov: MolotovInfo }
    | { Smoke: GrenadeInfo }
    | { Flashbang: GrenadeInfo }
    | { HeGrenade: GrenadeInfo }
    | { Decoy: GrenadeInfo };

export interface WeaponInfo {
    weapon: string;
    position: Vec3;
    ammo: Ammo;
}

export interface GrenadeInfo {
    entity: number;
    position: Vec3;
    name: string;
}

export interface InfernoInfo {
    entity: number;
    position: Vec3;
    hull: Vec3[];
}

export interface MolotovInfo {
    entity: number;
    position: Vec3;
    is_incendiary: boolean;
}

export type Vec2 = [x: number, y: number];
export type Vec3 = [x: number, y: number, z: number];
export type Ammo = [clip: number, reserve: number];

// todo: cross-check this
export enum Color {
    Blue = 0,
    Green = 1,
    Yellow = 2,
    Orange = 3,
    Purple = 4,
}
