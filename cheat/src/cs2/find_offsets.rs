use std::time::Instant;

use crate::{
    constants::cs2,
    cs2::{
        CS2,
        offsets::Offsets,
        signatures,
    },
};

impl CS2 {
    pub fn find_offsets(&self) -> Option<Offsets> {
        let start = Instant::now();
        let mut offsets = Offsets::default();

        offsets.library.client = self.process.module_base_address(cs2::CLIENT_LIB)?;
        offsets.library.engine = self.process.module_base_address(cs2::ENGINE_LIB)?;
        offsets.library.tier0 = self.process.module_base_address(cs2::TIER0_LIB)?;
        offsets.library.input = self.process.module_base_address(cs2::INPUT_LIB)?;
        offsets.library.sdl = self.process.module_base_address(cs2::SDL_LIB)?;
        offsets.library.schema = self.process.module_base_address(cs2::SCHEMA_LIB)?;

        let Some(resource_offset) = self
            .process
            .get_interface_offset(offsets.library.engine, "GameResourceServiceClientV0")
        else {
            utils::warn!("could not get offset for GameResourceServiceClient");
            return None;
        };
        offsets.interface.resource = resource_offset;

        let Some(cvar_address) = self
            .process
            .get_interface_offset(offsets.library.tier0, "VEngineCvar0")
        else {
            utils::warn!("could not get convar interface offset");
            return None;
        };
        offsets.interface.cvar = cvar_address;

        let Some(input_address) = self
            .process
            .get_interface_offset(offsets.library.input, "InputSystemVersion0")
        else {
            utils::warn!("could not get input interface offset");
            return None;
        };
        offsets.interface.input = input_address;

        let resolved = signatures::resolve(&self.process, &offsets.library)?;
        let globals = resolved.globals;
        let schema = resolved.schema;
        offsets.direct = globals.direct;
        offsets.interface.entity =
            self.process.read::<u64>(offsets.interface.resource + 0x50) + 0x10;

        offsets.direct.button_state = self.process.read::<u32>(
            self.process
                .get_interface_function(offsets.interface.input, 19)
                + 0x14,
        ) as u64;

        let Some(sdl_window) = self
            .process
            .get_module_export(offsets.library.sdl, "SDL_GetKeyboardFocus")
        else {
            utils::warn!("could not find sdl window offset");
            return None;
        };
        let sdl_window = self.process.get_relative_address(sdl_window, 0x02, 0x06);
        let sdl_window = self.process.read(sdl_window);
        offsets.direct.sdl_window = self.process.get_relative_address(sdl_window, 0x03, 0x07);

        offsets.convar.sensitivity = globals.sensitivity;
        if offsets.convar.sensitivity == 0 {
            offsets.convar.sensitivity = self
                .process
                .get_convar(offsets.interface.cvar, "sensitivity")
                .unwrap_or(0);
        }
        if offsets.convar.sensitivity == 0 {
            utils::debug!("sensitivity not found, using default");
        }

        offsets.convar.ffa = self
            .process
            .get_convar(offsets.interface.cvar, "mp_teammates_are_enemies")
            .unwrap_or(0);

        let client = schema.get_library(cs2::CLIENT_LIB)?;

        offsets.controller.steam_id = client.get("CBasePlayerController", "m_steamID")?;
        offsets.controller.name = client.get("CBasePlayerController", "m_iszPlayerName")?;
        offsets.controller.pawn = client.get("CBasePlayerController", "m_hPawn")?;
        offsets.controller.desired_fov = client.get("CBasePlayerController", "m_iDesiredFOV")?;
        offsets.controller.owner_entity = client.get("C_BaseEntity", "m_hOwnerEntity")?;
        offsets.controller.color = client.get("CCSPlayerController", "m_iCompTeammateColor")?;
        offsets.controller.action_tracking_services =
            client.get("CCSPlayerController", "m_pActionTrackingServices")?;

        offsets.pawn.health = client.get("C_BaseEntity", "m_iHealth")?;
        offsets.pawn.armor = client.get("C_CSPlayerPawn", "m_ArmorValue")?;
        offsets.pawn.team = client.get("C_BaseEntity", "m_iTeamNum")?;
        offsets.pawn.life_state = client.get("C_BaseEntity", "m_lifeState")?;
        offsets.pawn.fov_multiplier = client.get("C_BasePlayerPawn", "m_flFOVSensitivityAdjust")?;
        offsets.pawn.game_scene_node = client.get("C_BaseEntity", "m_pGameSceneNode")?;
        offsets.pawn.eye_offset = client.get("C_BaseModelEntity", "m_vecViewOffset")?;
        offsets.pawn.eye_angles = client.get("C_CSPlayerPawn", "m_angEyeAngles")?;
        offsets.pawn.velocity = client.get("C_BaseEntity", "m_vecAbsVelocity")?;
        offsets.pawn.flags = client.get("C_BaseEntity", "m_fFlags")?;
        offsets.pawn.shots_fired = client.get("C_CSPlayerPawn", "m_iShotsFired")?;
        offsets.pawn.view_angles = client.get("C_BasePlayerPawn", "v_angle")?;
        offsets.pawn.spotted_state = client.get("C_CSPlayerPawn", "m_entitySpottedState")?;
        offsets.pawn.crosshair_entity = client.get("C_CSPlayerPawn", "m_iIDEntIndex")?;
        offsets.pawn.is_scoped = client.get("C_CSPlayerPawn", "m_bIsScoped")?;
        offsets.pawn.flash_alpha = client.get("C_CSPlayerPawnBase", "m_flFlashMaxAlpha")?;
        offsets.pawn.flash_duration = client.get("C_CSPlayerPawnBase", "m_flFlashDuration")?;
        offsets.pawn.deathmatch_immunity = client.get("C_CSPlayerPawn", "m_bGunGameImmunity")?;

        offsets.pawn.camera_services = client.get("C_BasePlayerPawn", "m_pCameraServices")?;
        offsets.pawn.item_services = client.get("C_BasePlayerPawn", "m_pItemServices")?;
        offsets.pawn.weapon_services = client.get("C_BasePlayerPawn", "m_pWeaponServices")?;
        offsets.pawn.observer_services = client.get("C_BasePlayerPawn", "m_pObserverServices")?;
        offsets.pawn.aim_punch_services = client.get("C_CSPlayerPawn", "m_pAimPunchServices")?;

        offsets.game_scene_node.dormant = client.get("CGameSceneNode", "m_bDormant")?;
        offsets.game_scene_node.origin = client.get("CGameSceneNode", "m_vecAbsOrigin")?;
        offsets.game_scene_node.model_state = client.get("CSkeletonInstance", "m_modelState")?;

        offsets.model_state.skeleton_instance =
            client.get("CBodyComponentSkeletonInstance", "m_skeletonInstance")?;

        offsets.smoke.did_smoke_effect =
            client.get("C_SmokeGrenadeProjectile", "m_bDidSmokeEffect")?;
        offsets.smoke.smoke_color = client.get("C_SmokeGrenadeProjectile", "m_vSmokeColor")?;

        offsets.molotov.is_incendiary = client.get("C_MolotovProjectile", "m_bIsIncGrenade")?;

        offsets.inferno.is_burning = client.get("C_Inferno", "m_bFireIsBurning")?;
        offsets.inferno.fire_count = client.get("C_Inferno", "m_fireCount")?;
        offsets.inferno.fire_positions = client.get("C_Inferno", "m_firePositions")?;

        offsets.spotted_state.mask = client.get("EntitySpottedState_t", "m_bSpottedByMask")?;

        offsets.action_tracking.round_kills = client.get(
            "CCSPlayerController_ActionTrackingServices",
            "m_iNumRoundKills",
        )?;
        offsets.action_tracking.round_damage = client.get(
            "CCSPlayerController_ActionTrackingServices",
            "m_flTotalRoundDamageDealt",
        )?;

        offsets.camera_services.fov = client.get("CCSPlayerBase_CameraServices", "m_iFOV")?;

        offsets.item_services.has_defuser =
            client.get("CCSPlayer_ItemServices", "m_bHasDefuser")?;
        offsets.item_services.has_helmet = client.get("CCSPlayer_ItemServices", "m_bHasHelmet")?;

        offsets.weapon_services.active_weapon =
            client.get("CPlayer_WeaponServices", "m_hActiveWeapon")?;
        offsets.weapon_services.weapons = client.get("CPlayer_WeaponServices", "m_hMyWeapons")?;

        offsets.observer_services.target =
            client.get("CPlayer_ObserverServices", "m_hObserverTarget")?;

        offsets.aim_punch_services.aim_punch_cache = client
            .get_field("CCSPlayer_AimPunchServices", "m_aimPunchAngle")
            .or_else(|| {
                client
                    .get_field("CCSPlayer_AimPunchServices", "m_unpredictableBaseTick")
                    .map(|offset| offset - 0x18)
            })?;

        offsets.weapon.attribute_manager = client.get("C_EconEntity", "m_AttributeManager")?;
        offsets.weapon.item = client.get("C_AttributeContainer", "m_Item")?;
        offsets.weapon.clip_primary = client.get("C_BasePlayerWeapon", "m_iClip1")?;
        offsets.weapon.reserve_ammo = client.get("C_BasePlayerWeapon", "m_pReserveAmmo")?;

        offsets.econ_item_view.item_definition_index =
            client.get("C_EconItemView", "m_iItemDefinitionIndex")?;

        offsets.planted_c4.is_ticking = client.get("C_PlantedC4", "m_bBombTicking")?;
        offsets.planted_c4.blow_time = client.get("C_PlantedC4", "m_flC4Blow")?;
        offsets.planted_c4.being_defused = client.get("C_PlantedC4", "m_bBeingDefused")?;
        offsets.planted_c4.is_defused = client.get("C_PlantedC4", "m_bBombDefused")?;
        offsets.planted_c4.has_exploded = client.get("C_PlantedC4", "m_bHasExploded")?;
        offsets.planted_c4.defuse_time_left = client.get("C_PlantedC4", "m_flDefuseCountDown")?;

        offsets.entity_identity.size = client.get_class("CEntityIdentity")?.size();

        utils::debug!("offsets: {:?} ({:?})", offsets, Instant::now() - start);
        Some(offsets)
    }
}
