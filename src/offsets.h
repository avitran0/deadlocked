#ifndef OFFSETS_H
#define OFFSETS_H

#include "types.h"

typedef struct LibraryOffsets {
    u64 client;
    u64 engine;
    u64 tier0;
    u64 input;
    u64 sdl;
} LibraryOffsets;

typedef struct InterfaceOffsets {
    u64 convar;
    u64 resource;
    u64 entity;
    u64 player;
    u64 input;
} InterfaceOffsets;

typedef struct DirectOffsets {
    u64 local_controller;
    u64 button_state;
} DirectOffsets;

typedef struct ConvarOffsets {
    u64 sensitivity;
    u64 ffa;
} ConvarOffsets;

// todo: finish these offsets
typedef struct ControllerOffsets {
    u64 pawn;  // Pointer -> Pawn (m_hPawn)
} ControllerOffsets;

typedef struct PawnOffsets {
    u64 health;           // i32 (m_iHealth)
    u64 team;             // u8 (m_iTeamNum)
    u64 life_state;       // u8 (m_lifeState)
    u64 weapon;           // Pointer -> Weapon (m_pClippingWeapon)
    u64 fov_multiplier;   // f32 (m_flFOVSensitivityAdjust)
    u64 game_scene_node;  // Pointer -> GameSceneNode (m_pGameSceneNode)
    u64 eye_offset;       // Vec3 (m_vecViewOffset)
    u64 aim_punch_cache;  // Vector<Vec3> (m_aimPunchCache)
    u64 shots_fired;      // i32 (m_iShotsFired)
    u64 view_angles;      // Vec2 (v_angle)
    u64 spotted_state;    // SpottedState (m_entitySpottedState)
} PawnOffsets;

typedef struct GameSceneNodeOffsets {
    u64 dormant;      // bool (m_bDormant)
    u64 origin;       // Vec3 (m_vecAbsOrigin)
    u64 model_state;  // Pointer -> ModelState (m_modelState)
} GameSceneNodeOffsets;

typedef struct SpottedStateOffsets {
    u64 spotted; // bool (m_bSpotted)
    u64 mask;    // i32[2]? or u64? (m_bSpottedByMask)
} SpottedStateOffsets;

typedef struct Offsets {
    LibraryOffsets library;
    InterfaceOffsets interface;
    DirectOffsets direct;
    ConvarOffsets convars;

    ControllerOffsets controller;
    PawnOffsets pawn;
    GameSceneNodeOffsets game_scene_node;
    SpottedStateOffsets spotted_state;
} Offsets;

bool all_offsets_found(Offsets *offsets);

#endif
