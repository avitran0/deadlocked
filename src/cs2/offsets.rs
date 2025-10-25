// ------------------------------------------------------------------
// 0-deps, zero-cost, compile-time checked offset tables
// ------------------------------------------------------------------
#![allow(non_camel_case_types)]

/// Builds a plain-old-data struct plus its Default impl from a single
/// declarative list.  Memory layout stays identical
/// code; we only remove the boiler-plate.
macro_rules! offsets {
    (
        $(#[$outer:meta])*
        $vis:vis struct $Name:ident {
            $($field:ident : $ty:ty = $val:expr, $doc:literal),+ $(,)?
        }
    ) => {
        #[derive(Copy, Clone, Debug)]
        #[repr(C)]
        $(#[$outer])*
        $vis struct $Name {
            $(#[doc = $doc] pub $field: $ty),+
        }

        impl Default for $Name {
            #[inline]
            fn default() -> Self {
                Self { $($field: $val),+ }
            }
        }
    };
}

// ------------------------------------------------------------------
//  Library & interface blocks
// ------------------------------------------------------------------
offsets! {
    #[derive(Default)]
    pub struct LibraryOffsets {
        client: u64 = 0, "client.dll base address"
        engine: u64 = 0, "engine.dll base address"
        tier0:  u64 = 0, "tier0.dll base address"
        input:  u64 = 0, "inputsystem.dll base address"
        sdl:    u64 = 0, "SDL3.dll base address"
        schema: u64 = 0, "schemasystem.dll base address"
    }
}

offsets! {
    pub struct InterfaceOffsets {
        resource: u64 = 0, "CCSPlayerResource"
        entity:   u64 = 0, "CGameEntitySystem"
        cvar:     u64 = 0, "ICvar"
        input:    u64 = 0, "IInputSystem"
    }
}

// ------------------------------------------------------------------
//  World / global offsets
// ------------------------------------------------------------------
offsets! {
    pub struct DirectOffsets {
        local_player: u64 = 0, "LocalPlayer controller pointer"
        button_state: u64 = 0, "Global button state"
        view_matrix:  u64 = 0, "View-matrix array"
        sdl_window:   u64 = 0, "SDL_Window *"
        planted_c4:   u64 = 0, "C_PlantedC4 entity list"
        global_vars:  u64 = 0, "CGlobalVarsBase"
    }
}

offsets! {
    pub struct ConvarOffsets {
        ffa:         u64 = 0, "mp_teammates_are_enemies"
        sensitivity: u64 = 0, "sensitivity"
    }
}

// ------------------------------------------------------------------
//  Player controller
// ------------------------------------------------------------------
offsets! {
    pub struct PlayerControllerOffsets {
        steam_id:     u64 = 0, "m_steamID"
        name:         u64 = 0, "m_iszPlayerName"
        pawn:         u64 = 0, "m_hPawn"
        desired_fov:  u64 = 0, "m_iDesiredFOV"
        owner_entity: u64 = 0, "m_hOwnerEntity"
        color:        u64 = 0, "m_iCompTeammateColor"
    }
}

// ------------------------------------------------------------------
//  Pawn (the physical entity in the world)
// ------------------------------------------------------------------
offsets! {
    pub struct PawnOffsets {
        health:              u64 = 0, "m_iHealth"
        armor:               u64 = 0, "m_ArmorValue"
        team:                u64 = 0, "m_iTeamNum"
        life_state:          u64 = 0, "m_lifeState"
        weapon:              u64 = 0, "m_pClippingWeapon"
        fov_multiplier:      u64 = 0, "m_flFOVSensitivityAdjust"
        game_scene_node:     u64 = 0, "m_pGameSceneNode"
        eye_offset:          u64 = 0, "m_vecViewOffset"
        eye_angles:          u64 = 0, "m_angEyeAngles"
        velocity:            u64 = 0, "m_vecAbsVelocity"
        aim_punch_cache:     u64 = 0, "m_aimPunchCache"
        shots_fired:         u64 = 0, "m_iShotsFired"
        view_angles:         u64 = 0, "v_angle"
        spotted_state:       u64 = 0, "m_entitySpottedState"
        crosshair_entity:    u64 = 0, "m_iIDEntIndex"
        is_scoped:           u64 = 0, "m_bIsScoped"
        flash_alpha:         u64 = 0, "m_flFlashMaxAlpha"
        flash_duration:      u64 = 0, "m_flFlashDuration"
        deathmatch_immunity: u64 = 0, "m_bGunGameImmunity"
        camera_services:     u64 = 0, "m_pCameraServices"
        item_services:       u64 = 0, "m_pItemServices"
        weapon_services:     u64 = 0, "m_pWeaponServices"
        observer_services:   u64 = 0, "m_pObserverServices"
    }
}

// ------------------------------------------------------------------
//  Scene-graph & skeleton
// ------------------------------------------------------------------
offsets! {
    pub struct GameSceneNodeOffsets {
        dormant:    u64 = 0, "m_bDormant"
        origin:     u64 = 0, "m_vecAbsOrigin"
        model_state:u64 = 0, "m_modelState"
    }
}

offsets! {
    pub struct SkeletonInstanceOffsets {
        skeleton_instance: u64 = 0, "m_skeletonInstance"
    }
}

// ------------------------------------------------------------------
//  Grenades & environment
// ------------------------------------------------------------------
offsets! {
    pub struct SmokeOffsets {
        did_smoke_effect: u64 = 0, "m_bDidSmokeEffect"
        smoke_color:      u64 = 0, "m_vSmokeColor"
    }
}

offsets! {
    pub struct MolotovOffsets {
        is_incendiary: u64 = 0, "m_bIsIncGrenade"
    }
}

offsets! {
    pub struct InfernoOffsets {
        is_burning:     u64 = 0, "m_bFireIsBurning"
        fire_count:     u64 = 0, "m_fireCount"
        fire_positions: u64 = 0, "m_firePositions"
    }
}

// ------------------------------------------------------------------
//  Services & state helpers
// ------------------------------------------------------------------
offsets! {
    pub struct SpottedStateOffsets {
        spotted: u64 = 0, "m_bSpotted"
        mask:    u64 = 0, "m_bSpottedByMask"
    }
}

offsets! {
    pub struct CameraServicesOffsets {
        fov: u64 = 0, "m_iFOV"
    }
}

offsets! {
    pub struct ItemServicesOffsets {
        has_defuser: u64 = 0, "m_bHasDefuser"
        has_helmet:  u64 = 0, "m_bHasHelmet"
    }
}

offsets! {
    pub struct WeaponServicesOffsets {
        weapons: u64 = 0, "m_hMyWeapons"
    }
}

offsets! {
    pub struct ObserverServicesOffsets {
        target: u64 = 0, "m_hObserverTarget"
    }
}

// ------------------------------------------------------------------
//  Weapons & bomb
// ------------------------------------------------------------------
offsets! {
    pub struct WeaponOffsets {
        attribute_manager:     u64 = 0, "m_AttributeManager"
        item:                  u64 = 0, "m_Item"
        item_definition_index: u64 = 0, "m_iItemDefinitionIndex"
    }
}

offsets! {
    pub struct PlantedC4Offsets {
        is_ticking:      u64 = 0, "m_bBombTicking"
        blow_time:       u64 = 0, "m_flC4Blow"
        being_defused:   u64 = 0, "m_bBeingDefused"
        is_defused:      u64 = 0, "m_bBombDefused"
        has_exploded:    u64 = 0, "m_bHasExploded"
        defuse_time_left:u64 = 0, "m_flDefuseCountDown"
    }
}

offsets! {
    pub struct EntityIdentityOffsets {
        size: i32 = 0, "sizeof(CEntityIdentity)"
    }
}

// ------------------------------------------------------------------
//  Top-level container
// ------------------------------------------------------------------
#[derive(Copy, Clone, Debug, Default)]
pub struct Offsets {
    pub library: LibraryOffsets,
    pub interface: InterfaceOffsets,
    pub direct: DirectOffsets,
    pub convar: ConvarOffsets,
    pub controller: PlayerControllerOffsets,
    pub pawn: PawnOffsets,
    pub game_scene_node: GameSceneNodeOffsets,
    pub skeleton: SkeletonInstanceOffsets,
    pub smoke: SmokeOffsets,
    pub molotov: MolotovOffsets,
    pub inferno: InfernoOffsets,
    pub spotted_state: SpottedStateOffsets,
    pub camera_services: CameraServicesOffsets,
    pub item_services: ItemServicesOffsets,
    pub weapon_services: WeaponServicesOffsets,
    pub observer_services: ObserverServicesOffsets,
    pub weapon: WeaponOffsets,
    pub planted_c4: PlantedC4Offsets,
    pub entity_identity: EntityIdentityOffsets,
}
