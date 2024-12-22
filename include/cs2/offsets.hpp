#pragma once

#include "types.hpp"

struct LibraryOffsets {
    u64 client = 0;
    u64 engine = 0;
    u64 tier0 = 0;
    u64 input = 0;
    u64 sdl = 0;
    u64 matchmaking = 0;
};

struct InterfaceOffsets {
    u64 resource = 0;
    u64 entity = 0;
    u64 cvar = 0;
    u64 player = 0;
    u64 input = 0;
};

struct DirectOffsets {
    u64 local_player = 0;
    u64 button_state = 0;
    u64 view_matrix = 0;
    u64 sdl_window = 0;
};

struct ConvarOffsets {
    u64 ffa = 0;
    u64 sensitivity = 0;
};

struct PlayerControllerOffsets {
    u64 name = 0;          // Pointer -> String (m_sSanitizedPlayerName)
    u64 pawn = 0;          // Pointer -> Pawn (m_hPawn)
    u64 owner_entity = 0;  // Handle -> Pawn (m_hOwnerEntity)

    bool AllFound() const { return name && pawn && owner_entity; }
};

struct PawnOffsets {
    u64 health = 0;             // i32 (m_iHealth)
    u64 armor = 0;              // i32 (m_ArmorValue)
    u64 team = 0;               // i32 (m_iTeamNum)
    u64 life_state = 0;         // i32 (m_lifeState)
    u64 weapon = 0;             // Pointer -> WeaponBase (m_pClippingWeapon)
    u64 fov_multiplier = 0;     // f32 (m_flFOVSensitivityAdjust)
    u64 game_scene_node = 0;    // Pointer -> GameSceneNode (m_pGameSceneNode)
    u64 eye_offset = 0;         // Vec3 (m_vecViewOffset)
    u64 velocity = 0;           // Vec3 (m_vecVelocity)
    u64 aim_punch_cache = 0;    // Vector<Vec3> (m_aimPunchCache)
    u64 shots_fired = 0;        // i32 (m_iShotsFired)
    u64 view_angles = 0;        // Vec2 (v_angle)
    u64 spotted_state = 0;      // SpottedState (m_entitySpottedState)
    u64 observer_services = 0;  // Pointer -> ObserverServices (m_pObserverServices)

    bool AllFound() const {
        return health && armor && team && life_state && weapon && fov_multiplier && game_scene_node && eye_offset &&
               aim_punch_cache && shots_fired && view_angles && spotted_state && observer_services;
    }
};

struct GameSceneNodeOffsets {
    u64 dormant = 0;      // bool (m_bDormant)
    u64 origin = 0;       // Vec3 (m_vecAbsOrigin)
    u64 model_state = 0;  // Pointer -> ModelState (m_modelState)

    bool AllFound() const { return dormant && origin && model_state; }
};

struct SpottedStateOffsets {
    u64 spotted = 0;  // bool (m_bSpotted)
    u64 mask = 0;     // i32[2] or u64? (m_bSpottedByMask)

    bool AllFound() const { return spotted && mask; }
};

struct ObserverServiceOffsets {
    u64 target = 0;  // pointer -> Pawn (m_hObserverTarget)

    bool AllFound() const { return target; }
};

struct Offsets {
    LibraryOffsets library;
    InterfaceOffsets interface;
    DirectOffsets direct;
    ConvarOffsets convar;
    PlayerControllerOffsets controller;
    PawnOffsets pawn;
    GameSceneNodeOffsets game_scene_node;
    SpottedStateOffsets spotted_state;
    ObserverServiceOffsets observer_service;

    bool AllFound() const {
        return controller.AllFound() && pawn.AllFound() && game_scene_node.AllFound() && spotted_state.AllFound() &&
               observer_service.AllFound();
    }
};