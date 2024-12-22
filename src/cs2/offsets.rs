#[derive(Debug, Default)]
pub struct LibraryOffsets {
    pub client: u64,
    pub engine: u64,
    pub tier0: u64,
    pub input: u64,
    pub sdl: u64,
    pub matchmaking: u64,
}

#[derive(Debug, Default)]
pub struct InterfaceOffsets {
    pub resource: u64,
    pub entity: u64,
    pub cvar: u64,
    pub player: u64,
    pub input: u64,
}

#[derive(Debug, Default)]
pub struct DirectOffsets {
    pub local_player: u64,
    pub button_state: u64,
    pub view_matrix: u64,
    pub sdl_window: u64,
    pub planted_c4: u64,
}

#[derive(Debug, Default)]
pub struct ConvarOffsets {
    pub ffa: u64,
    pub sensitivity: u64,
}

#[derive(Debug, Default)]
pub struct PlayerControllerOffsets {
    pub name: u64,         // Pointer -> String (m_sSanitizedPlayerName)
    pub pawn: u64,         // Pointer -> Pawn (m_hPawn)
    pub owner_entity: u64, // Pointer -> Entity (m_hOwnerEntity)
}

impl PlayerControllerOffsets {
    pub fn all_found(&self) -> bool {
        self.name != 0 && self.pawn != 0 && self.owner_entity != 0
    }
}

#[derive(Debug, Default)]
pub struct PawnOffsets {
    pub health: u64,            // i32 (m_iHealth)
    pub armor: u64,             // i32 (m_ArmorValue)
    pub team: u64,              // i32 (m_iTeamNum)
    pub life_state: u64,        // i32 (m_lifeState)
    pub weapon: u64,            // Pointer -> WeaponBase (m_pClippingWeapon)
    pub fov_multiplier: u64,    // f32 (m_flFOVSensitivityAdjust)
    pub game_scene_node: u64,   // Pointer -> GameSceneNode (m_pGameSceneNode)
    pub eye_offset: u64,        // Vec3 (m_vecViewOffset)
    pub velocity: u64,          // Vec3 (m_vecVelocity)
    pub aim_punch_cache: u64,   // Vector<Vec3> (m_aimPunchCache)
    pub shots_fired: u64,       // i32 (m_iShotsFired)
    pub view_angles: u64,       // Vec2 (v_angle)
    pub spotted_state: u64,     // SpottedState (m_entitySpottedState)
    pub observer_services: u64, // Pointer -> ObserverServices (m_pObserverServices)
}

impl PawnOffsets {
    pub fn all_found(&self) -> bool {
        self.health != 0
            && self.armor != 0
            && self.team != 0
            && self.life_state != 0
            && self.weapon != 0
            && self.fov_multiplier != 0
            && self.game_scene_node != 0
            && self.eye_offset != 0
            && self.aim_punch_cache != 0
            && self.shots_fired != 0
            && self.view_angles != 0
            && self.spotted_state != 0
            && self.observer_services != 0
    }
}

#[derive(Debug, Default)]
pub struct GameSceneNodeOffsets {
    pub dormant: u64,     // bool (m_bDormant)
    pub origin: u64,      // Vec3 (m_vecAbsOrigin)
    pub model_state: u64, // Pointer -> ModelState (m_modelState)
}

impl GameSceneNodeOffsets {
    pub fn all_found(&self) -> bool {
        self.dormant != 0 && self.origin != 0 && self.model_state != 0
    }
}

#[derive(Debug, Default)]
pub struct SpottedStateOffsets {
    pub spotted: u64, // bool (m_bSpotted)
    pub mask: u64,    // i32[2] or u64? (m_bSpottedByMask)
}

impl SpottedStateOffsets {
    pub fn all_found(&self) -> bool {
        self.spotted != 0 && self.mask != 0
    }
}

#[derive(Debug, Default)]
pub struct ObserverServiceOffsets {
    pub target: u64, // pointer -> Pawn (m_hObserverTarget)
}

impl ObserverServiceOffsets {
    pub fn all_found(&self) -> bool {
        self.target != 0
    }
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
    pub spotted_state: SpottedStateOffsets,
    pub observer_service: ObserverServiceOffsets,
}

impl Offsets {
    pub fn all_found(&self) -> bool {
        self.controller.all_found()
            && self.pawn.all_found()
            && self.game_scene_node.all_found()
            && self.spotted_state.all_found()
            && self.observer_service.all_found()
    }
}
