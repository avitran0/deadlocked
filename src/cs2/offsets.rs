#[derive(Debug, Default)]
pub struct LibraryOffsets {
    pub client: usize,
    pub engine: usize,
    pub tier0: usize,
    pub input: usize,
    pub sdl: usize,
    pub schema: usize,
}

#[derive(Debug, Default)]
pub struct InterfaceOffsets {
    pub resource: usize,
    pub entity: usize,
    pub cvar: usize,
    pub input: usize,
}

#[derive(Debug, Default)]
pub struct DirectOffsets {
    pub local_player: usize,
    pub button_state: usize,
    pub view_matrix: usize,
    pub sdl_window: usize,
    pub planted_c4: usize,
    pub global_vars: usize,
    pub vphys_world: usize,
}

#[derive(Debug, Default)]
pub struct ConvarOffsets {
    pub ffa: usize,
    pub sensitivity: usize,
}

#[derive(Debug, Default)]
pub struct PlayerControllerOffsets {
    pub steam_id: usize,                 // u64 (m_steamID)
    pub name: usize,                     // Pointer -> String (m_iszPlayerName)
    pub pawn: usize,                     // Handle -> Pawn (m_hPawn)
    pub desired_fov: usize,              // u32 (m_iDesiredFOV)
    pub owner_entity: usize,             // i32 (h_pOwnerEntity)
    pub color: usize,                    // i32 (m_iCompTeammateColor)
    pub action_tracking_services: usize, // Pointer -> ActionTrackingServices (m_pActionTrackingServices)
}

#[derive(Debug, Default)]
pub struct PawnOffsets {
    pub health: usize,              // i32 (m_iHealth)
    pub armor: usize,               // i32 (m_ArmorValue)
    pub team: usize,                // i32 (m_iTeamNum)
    pub life_state: usize,          // i32 (m_lifeState)
    pub fov_multiplier: usize,      // f32 (m_flFOVSensitivityAdjust)
    pub game_scene_node: usize,     // Pointer -> GameSceneNode (m_pGameSceneNode)
    pub eye_offset: usize,          // Vec3 (m_vecViewOffset)
    pub eye_angles: usize,          // Vec3 (m_angEyeAngles)
    pub velocity: usize,            // Vec3 (m_vecAbsVelocity)
    pub flags: usize,               // i32 (m_fFlags)
    pub shots_fired: usize,         // i32 (m_iShotsFired)
    pub view_angles: usize,         // Vec2 (v_angle)
    pub spotted_state: usize,       // SpottedState (m_entitySpottedState)
    pub crosshair_entity: usize,    // EntityIndex (m_iIDEntIndex)
    pub is_scoped: usize,           // bool (m_bIsScoped)
    pub flash_alpha: usize,         // f32 (m_flFlashMaxAlpha)
    pub flash_duration: usize,      // f32 (m_flFlashDuration)
    pub deathmatch_immunity: usize, // bool (m_bGunGameImmunity)
    pub camera_services: usize,     // Pointer -> CameraServices (m_pCameraServices)
    pub item_services: usize,       // Pointer -> ItemServices (m_pItemServices)
    pub weapon_services: usize,     // Pointer -> WeaponSercies (m_pWeaponServices)
    pub observer_services: usize,   // Pointer -> ObserverServices (m_pObserverServices)
    pub aim_punch_services: usize,  // Pointer -> AimPunchServices (m_pAimPunchServices)
}

#[derive(Debug, Default)]
pub struct GameSceneNodeOffsets {
    pub dormant: usize,     // bool (m_bDormant)
    pub origin: usize,      // Vec3 (m_vecAbsOrigin)
    pub model_state: usize, // Pointer -> ModelState (m_modelState)
}

#[derive(Debug, Default)]
pub struct ModelState {
    pub skeleton_instance: usize, // CSkeletonInstance (m_skeletonInstance)
}

#[derive(Debug, Default)]
pub struct SmokeOffsets {
    pub did_smoke_effect: usize, // bool (m_bDidSmokeEffect)
    pub smoke_color: usize,      // Vec3 (m_vSmokeColor)
}

#[derive(Debug, Default)]
pub struct MolotovOffsets {
    pub is_incendiary: usize, // bool (m_bIsIncGrenade)
}

#[derive(Debug, Default)]
pub struct InfernoOffsets {
    pub is_burning: usize,     // bool[64] (m_bFireIsBurning)
    pub fire_count: usize,     // i32 (m_fireCount)
    pub fire_positions: usize, // Vec3[64] (m_firePositions)
}

#[derive(Debug, Default)]
pub struct SpottedStateOffsets {
    pub mask: usize, // i32[2] or u64? (m_bSpottedByMask)
}

#[derive(Debug, Default)]
pub struct ActionTrackingServicesOffsets {
    pub round_kills: usize,  // i32 (m_iNumRoundKills)
    pub round_damage: usize, // f32 (m_flTotalRoundDamageDealt)
}

#[derive(Debug, Default)]
pub struct CameraServicesOffsets {
    pub fov: usize, // u32 (m_iFOV)
}

#[derive(Debug, Default)]
pub struct ItemServicesOffsets {
    pub has_defuser: usize, // bool (m_bHasDefuser)
    pub has_helmet: usize,  // bool (m_bHasHelmet)
}

#[derive(Debug, Default)]
pub struct WeaponServicesOffsets {
    pub active_weapon: usize, // Handle<Weapon> (m_hActiveWeapon)
    pub weapons: usize,       // Pointer -> Vec<Pointer -> Weapon> (m_hMyWeapons)
}

#[derive(Debug, Default)]
pub struct ObserverServicesOffsets {
    pub target: usize, // Handle -> BaseEntity (m_hObserverTarget)
}

#[derive(Debug, Default)]
pub struct AimPunchSerciesOffsets {
    pub aim_punch_cache: usize, // Vec<Vec3> (m_unpredictableBaseTick - 0x18)
}

#[derive(Debug, Default)]
pub struct WeaponOffsets {
    pub attribute_manager: usize, // AttributeContainer (m_AttributeManager)
    pub item: usize,              // EconItemView (m_Item)
    pub clip_primary: usize,      // i32 (m_iClip1)
    pub reserve_ammo: usize,      // i32[2] (m_pReserveAmmo)
}

#[derive(Debug, Default)]
pub struct EconItemViewOffsets {
    pub item_definition_index: usize, // u16 (m_iItemDefinitionIndex)
}

#[derive(Debug, Default)]
pub struct PlantedC4Offsets {
    pub is_ticking: usize,       // bool (m_bBombTicking)
    pub blow_time: usize,        // f32 (m_flC4Blow)
    pub being_defused: usize,    // bool (m_bBeingDefused)
    pub is_defused: usize,       // bool (m_bBombDefused)
    pub has_exploded: usize,     // bool (m_bHasExploded)
    pub defuse_time_left: usize, // u64 (m_flDefuseCountDown)
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
    pub model_state: ModelState,
    pub smoke: SmokeOffsets,
    pub molotov: MolotovOffsets,
    pub inferno: InfernoOffsets,
    pub spotted_state: SpottedStateOffsets,
    pub action_tracking: ActionTrackingServicesOffsets,
    pub camera_services: CameraServicesOffsets,
    pub item_services: ItemServicesOffsets,
    pub weapon_services: WeaponServicesOffsets,
    pub observer_services: ObserverServicesOffsets,
    pub aim_punch_services: AimPunchSerciesOffsets,
    pub weapon: WeaponOffsets,
    pub econ_item_view: EconItemViewOffsets,
    pub planted_c4: PlantedC4Offsets,
    pub entity_identity: EntityIdentityOffsets,
}
