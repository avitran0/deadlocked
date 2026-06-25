export function defaultData(): Data {
    return {
        in_game: false,
        is_ffa: false,
        weapon: {},
        players: [],
        friendlies: [],
        local_player: {
            steam_id: 0,
            health: 0,
            armor: 0,
            position: [],
            name: "",
            weapon: {},
            ammo: [],
            has_defuser: false,
            has_helmet: false,
            has_bomb: false,
            visible: false,
            color: 0,
            rotation: 0,
        },
        entities: [],
        bomb: { planted: false, timer: 0, being_defused: false, position: [], defuse_remain_time: 0 },
        map_name: "de_dust2",
    };
}

// todo: C_CSGameRules m_vMinimapMins/m_vMinimapMaxs?

export interface Data {
    in_game: boolean;
    is_ffa: boolean;
    weapon: Weapon;
    players: PlayerData[];
    friendlies: PlayerData[];
    local_player: PlayerData;
    entities: EntityInfo[];
    bomb: BombData;
    map_name: string;
}

export interface PlayerData {
    steam_id: number;
    health: number;
    armor: number;
    // vec3 serialization?
    position: Vec3;
    name: string;
    weapon: Weapon;
    // tuple serialization?
    ammo: number[];
    has_defuser: boolean;
    has_helmet: boolean;
    has_bomb: boolean;
    visible: boolean;
    color: number;
    rotation: number;
}

export interface EntityInfo {}

export interface Weapon {}

export interface BombData {
    planted: boolean;
    timer: number;
    being_defused: boolean;
    position: Vec3;
    defuse_remain_time: number;
}

export type Vec3 = number[];
