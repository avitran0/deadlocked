#[derive(Debug, Default)]
pub struct LibraryOffsets {
    pub client: u64,
    pub engine: u64,
    pub tier0: u64,
    pub input: u64,
    pub sdl: u64,
    pub schema: u64,
}

#[derive(Debug, Default)]
pub struct InterfaceOffsets {
    pub resource: u64,
    pub entity: u64,
    pub cvar: u64,
    pub input: u64,
}

#[derive(Debug, Default)]
pub struct DirectOffsets {
    pub local_player: u64,
    pub button_state: u64,
    pub view_matrix: u64,
    pub sdl_window: u64,
    pub planted_c4: u64,
    pub global_vars: u64,
}

#[derive(Debug, Default)]
pub struct ConvarOffsets {
    pub ffa: u64,
    pub sensitivity: u64,
}

#[derive(Debug, Default)]
pub struct PlayerControllerOffsets {
    pub steam_id: u64,     // u64 (m_steamID)
    pub name: u64,         // Pointer -> String (m_iszPlayerName)
    pub pawn: u64,         // Handle -> Pawn (m_hPawn)
    pub desired_fov: u64,  // u32 (m_iDesiredFOV)
    pub owner_entity: u64, // i32 (h_pOwnerEntity)
    pub color: u64,        // i32 (m_iCompTeammateColor)
}

#[derive(Debug, Default)]
pub struct PawnOffsets {
    pub health: u64,              // i32 (m_iHealth)
    pub armor: u64,               // i32 (m_ArmorValue)
    pub team: u64,                // i32 (m_iTeamNum)
    pub life_state: u64,          // i32 (m_lifeState)
    pub weapon: u64,              // Pointer -> WeaponBase (m_pClippingWeapon)
    pub fov_multiplier: u64,      // f32 (m_flFOVSensitivityAdjust)
    pub game_scene_node: u64,     // Pointer -> GameSceneNode (m_pGameSceneNode)
    pub eye_offset: u64,          // Vec3 (m_vecViewOffset)
    pub eye_angles: u64,          // Vec3 (m_angEyeAngles)
    pub velocity: u64,            // Vec3 (m_vecAbsVelocity)
    pub aim_punch_cache: u64,     // Vector<Vec3> (m_aimPunchCache)
    pub shots_fired: u64,         // i32 (m_iShotsFired)
    pub view_angles: u64,         // Vec2 (v_angle)
    pub spotted_state: u64,       // SpottedState (m_entitySpottedState)
    pub crosshair_entity: u64,    // EntityIndex (m_iIDEntIndex)
    pub is_scoped: u64,           // bool (m_bIsScoped)
    pub flash_alpha: u64,         // f32 (m_flFlashMaxAlpha)
    pub flash_duration: u64,      // f32 (m_flFlashDuration)
    pub deathmatch_immunity: u64, // bool (m_bGunGameImmunity)
    pub camera_services: u64,     // Pointer -> CameraServices (m_pCameraServices)
    pub item_services: u64,       // Pointer -> ItemServices (m_pItemServices)
    pub weapon_services: u64,     // Pointer -> WeaponSercies (m_pWeaponServices)
    pub observer_services: u64,   // Pointer -> ObserverServices (m_pObserverServices)
}

#[derive(Debug, Default)]
pub struct GameSceneNodeOffsets {
    pub dormant: u64,     // bool (m_bDormant)
    pub origin: u64,      // Vec3 (m_vecAbsOrigin)
    pub model_state: u64, // Pointer -> ModelState (m_modelState)
}

#[derive(Debug, Default)]
pub struct SkeletonInstanceOffsets {
    pub skeleton_instance: u64, // CSkeletonInstance (m_skeletonInstance)
}

#[derive(Debug, Default)]
pub struct SmokeOffsets {
    pub did_smoke_effect: u64, // bool (m_bDidSmokeEffect)
    pub smoke_color: u64,      // Vec3 (m_vSmokeColor)
}

#[derive(Debug, Default)]
pub struct MolotovOffsets {
    pub is_incendiary: u64, // bool (m_bIsIncGrenade)
}

#[derive(Debug, Default)]
pub struct InfernoOffsets {
    pub is_burning: u64,     // bool[64] (m_bFireIsBurning)
    pub fire_count: u64,     // i32 (m_fireCount)
    pub fire_positions: u64, // Vec3[64] (m_firePositions)
}

#[derive(Debug, Default)]
pub struct SpottedStateOffsets {
    pub spotted: u64, // bool (m_bSpotted)
    pub mask: u64,    // i32[2] or u64? (m_bSpottedByMask)
}

#[derive(Debug, Default)]
pub struct CameraServicesOffsets {
    pub fov: u64, // u32 (m_iFOV)
}

#[derive(Debug, Default)]
pub struct ItemServicesOffsets {
    pub has_defuser: u64, // bool (m_bHasDefuser)
    pub has_helmet: u64,  // bool (m_bHasHelmet)
}

#[derive(Debug, Default)]
pub struct WeaponServicesOffsets {
    pub weapons: u64, // Pointer -> Vec<Pointer -> Weapon> (m_hMyWeapons)
}

#[derive(Debug, Default)]
pub struct ObserverServicesOffsets {
    pub target: u64, // Handle -> BaseEntity (m_hObserverTarget)
}

#[derive(Debug, Default)]
pub struct WeaponOffsets {
    pub attribute_manager: u64,     // AttributeContainer (m_AttributeManager)
    pub item: u64,                  // EIconItemView (m_Item)
    pub item_definition_index: u64, // u16 (m_iItemDefinitionIndex)
}

#[derive(Debug, Default)]
pub struct PlantedC4Offsets {
    pub is_ticking: u64,       // bool (m_bBombTicking)
    pub blow_time: u64,        // f32 (m_flC4Blow)
    pub being_defused: u64,    // bool (m_bBeingDefused)
    pub is_defused: u64,       // bool (m_bBombDefused)
    pub has_exploded: u64,     // bool (m_bHasExploded)
    pub defuse_time_left: u64, // u64 (m_flDefuseCountDown)
}

#[derive(Debug, Default)]
pub struct EntityIdentityOffsets {
    pub size: i32,
}

#[derive(Debug, Default)]
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
