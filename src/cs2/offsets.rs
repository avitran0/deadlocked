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
    pub planted_c4: u64,
    pub game_types: u64,
    pub sdl_window: u64,
}

#[derive(Debug, Default)]
pub struct ConvarOffsets {
    pub ffa: u64,
    pub sensitivity: u64,
}

#[derive(Debug, Default)]
pub struct PlayerControllerOffsets {
    pub name: u64,        // Pointer -> String (m_sSanitizedPlayerName)
    pub pawn: u64,        // Pointer -> Pawn (m_hPawn)
    pub desired_fov: u64, // u32 (m_iDesiredFOV)
}

impl PlayerControllerOffsets {
    pub fn all_found(&self) -> bool {
        self.name != 0 && self.pawn != 0 && self.desired_fov != 0
    }
}

#[derive(Debug, Default)]
pub struct PawnOffsets {
    pub health: u64,          // i32 (m_iHealth)
    pub armor: u64,           // i32 (m_ArmorValue)
    pub team: u64,            // i32 (m_iTeamNum)
    pub life_state: u64,      // i32 (m_lifeState)
    pub weapon: u64,          // Pointer -> WeaponBase (m_pClippingWeapon)
    pub fov_multiplier: u64,  // f32 (m_flFOVSensitivityAdjust)
    pub game_scene_node: u64, // Pointer -> GameSceneNode (m_pGameSceneNode)
    pub eye_offset: u64,      // Vec3 (m_vecViewOffset)
    pub velocity: u64,        // Vec3 (m_vecAbsVelocity)
    pub aim_punch_cache: u64, // Vector<Vec3> (m_aimPunchCache)
    pub shots_fired: u64,     // i32 (m_iShotsFired)
    pub view_angles: u64,     // Vec2 (v_angle)
    pub spotted_state: u64,   // SpottedState (m_entitySpottedState)
    pub glow: u64,            // Glow (m_Glow)
    pub flash_alpha: u64,     // f32 (m_flFlashMaxAlpha)
    pub flash_duration: u64,  // f32 (m_flFlashDuration)
    pub camera_services: u64, // Pointer -> CameraServices (m_pCameraServices)
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
            && self.glow != 0
            && self.flash_alpha != 0
            && self.flash_duration != 0
            && self.camera_services != 0
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
pub struct BombOffsets {
    pub is_ticking: u64, // bool (m_bBombTicking)
    pub bomb_site: u64,  // i32 (m_nBombSite)
    pub blow_time: u64,  // u32? (m_flC4Blow)
}

impl BombOffsets {
    pub fn all_found(&self) -> bool {
        self.is_ticking != 0 && self.bomb_site != 0 && self.blow_time != 0
    }
}

#[derive(Debug, Default)]
pub struct GlowOffsets {
    pub is_glowing: u64,     // bool (m_bGlowing)
    pub glow_type: u64,      // i32 (m_iGlowType)
    pub color_override: u64, // Color (m_glowColorOverride)
}

impl GlowOffsets {
    pub fn all_found(&self) -> bool {
        self.is_glowing != 0 && self.glow_type != 0 && self.color_override != 0
    }
}

#[derive(Debug, Default)]
pub struct CameraServicesOffsets {
    pub fov: u64, // u32 (m_iFOV)
}

impl CameraServicesOffsets {
    pub fn all_found(&self) -> bool {
        self.fov != 0
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
    pub bomb: BombOffsets,
    pub glow: GlowOffsets,
    pub camera_services: CameraServicesOffsets,
}

impl Offsets {
    pub fn all_found(&self) -> bool {
        self.controller.all_found()
            && self.pawn.all_found()
            && self.game_scene_node.all_found()
            && self.spotted_state.all_found()
            && self.bomb.all_found()
            && self.glow.all_found()
            && self.camera_services.all_found()
    }
}
