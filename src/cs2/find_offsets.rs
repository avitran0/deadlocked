use std::time::Instant;

use utils::log;

use crate::{
    constants::cs2,
    cs2::{CS2, offsets::Offsets, schema::Schema},
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
            log::warn!("could not get offset for GameResourceServiceClient");
            return None;
        };
        offsets.interface.resource = resource_offset;

        offsets.interface.entity =
            self.process.read::<u64>(offsets.interface.resource + 0x50) + 0x10;

        let Some(cvar_address) = self
            .process
            .get_interface_offset(offsets.library.tier0, "VEngineCvar0")
        else {
            log::warn!("could not get convar interface offset");
            return None;
        };
        offsets.interface.cvar = cvar_address;
        let Some(input_address) = self
            .process
            .get_interface_offset(offsets.library.input, "InputSystemVersion0")
        else {
            log::warn!("could not get input interface offset");
            return None;
        };
        offsets.interface.input = input_address;

        let Some(local_player) = self
            .process
            .scan("48 83 3D ? ? ? ? 00 0F 95 C0 C3", offsets.library.client)
        else {
            log::warn!("could not find local player offset");
            return None;
        };
        offsets.direct.local_player = self.process.get_relative_address(local_player, 0x03, 0x08);
        offsets.direct.button_state = self.process.read::<u32>(
            self.process
                .get_interface_function(offsets.interface.input, 19)
                + 0x14,
        ) as u64;

        let Some(view_matrix) = self
            .process
            .scan("C6 83 ? ? 00 00 01 4C 8D 05", offsets.library.client)
        else {
            log::warn!("could not find view matrix offset");
            return None;
        };

        offsets.direct.view_matrix =
            self.process
                .get_relative_address(view_matrix + 0x0A, 0x0, 0x04);

        let Some(sdl_window) = self
            .process
            .get_module_export(offsets.library.sdl, "SDL_GetKeyboardFocus")
        else {
            log::warn!("could not find sdl window offset");
            return None;
        };
        let sdl_window = self.process.get_relative_address(sdl_window, 0x02, 0x06);
        let sdl_window = self.process.read(sdl_window);
        offsets.direct.sdl_window = self.process.get_relative_address(sdl_window, 0x03, 0x07);

        let Some(planted_c4) = self.process.scan(
            "48 8D 35 ? ? ? ? 66 0F EF C0 C6 05 ? ? ? ? 01 48 8D 3D",
            offsets.library.client,
        ) else {
            log::warn!("could not find planted c4 offset");
            return None;
        };
        offsets.direct.planted_c4 = self.process.get_relative_address(planted_c4, 0x03, 0x0E);

        // xref "lobby_mapveto"
        let Some(global_vars) = self.process.scan(
            "48 8D 05 ? ? ? ? 48 8B 00 8B 48 ? E9",
            offsets.library.client,
        ) else {
            log::warn!("could not find global vars offset");
            return None;
        };
        offsets.direct.global_vars = self.process.get_relative_address(global_vars, 0x03, 0x07);

        let Some(ffa_address) = self
            .process
            .get_convar(offsets.interface.cvar, "mp_teammates_are_enemies")
        else {
            log::warn!("could not get mp_tammates_are_enemies convar offset");
            return None;
        };
        offsets.convar.ffa = ffa_address;
        let Some(sensitivity_address) = self
            .process
            .get_convar(offsets.interface.cvar, "sensitivity")
        else {
            log::warn!("could not get sensitivity convar offset");
            return None;
        };
        offsets.convar.sensitivity = sensitivity_address;

        let schema = Schema::new(&self.process, offsets.library.schema)?;
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
        offsets.pawn.weapon = client.get("C_CSPlayerPawn", "m_pClippingWeapon")?;
        offsets.pawn.fov_multiplier = client.get("C_BasePlayerPawn", "m_flFOVSensitivityAdjust")?;
        offsets.pawn.game_scene_node = client.get("C_BaseEntity", "m_pGameSceneNode")?;
        offsets.pawn.eye_offset = client.get("C_BaseModelEntity", "m_vecViewOffset")?;
        offsets.pawn.eye_angles = client.get("C_CSPlayerPawn", "m_angEyeAngles")?;
        offsets.pawn.velocity = client.get("C_BaseEntity", "m_vecAbsVelocity")?;
        offsets.pawn.flags = client.get("C_BaseEntity", "m_fFlags")?;
        offsets.pawn.aim_punch_cache = client.get("C_CSPlayerPawn", "m_aimPunchTickFraction")? + 8;
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

        offsets.game_scene_node.dormant = client.get("CGameSceneNode", "m_bDormant")?;
        offsets.game_scene_node.origin = client.get("CGameSceneNode", "m_vecAbsOrigin")?;
        offsets.game_scene_node.model_state = client.get("CSkeletonInstance", "m_modelState")?;

        offsets.skeleton.skeleton_instance =
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

        offsets.weapon_services.weapons = client.get("CPlayer_WeaponServices", "m_hMyWeapons")?;

        offsets.observer_services.target =
            client.get("CPlayer_ObserverServices", "m_hObserverTarget")?;

        offsets.weapon.attribute_manager = client.get("C_EconEntity", "m_AttributeManager")?;
        offsets.weapon.item = client.get("C_AttributeContainer", "m_Item")?;
        offsets.weapon.item_definition_index =
            client.get("C_EconItemView", "m_iItemDefinitionIndex")?;

        offsets.planted_c4.is_ticking = client.get("C_PlantedC4", "m_bBombTicking")?;
        offsets.planted_c4.blow_time = client.get("C_PlantedC4", "m_flC4Blow")?;
        offsets.planted_c4.being_defused = client.get("C_PlantedC4", "m_bBeingDefused")?;
        offsets.planted_c4.is_defused = client.get("C_PlantedC4", "m_bBombDefused")?;
        offsets.planted_c4.has_exploded = client.get("C_PlantedC4", "m_bHasExploded")?;
        offsets.planted_c4.defuse_time_left = client.get("C_PlantedC4", "m_flDefuseCountDown")?;

        offsets.entity_identity.size = client.get_class("CEntityIdentity")?.size();

        log::debug!("offsets: {:?} ({:?})", offsets, Instant::now() - start);
        Some(offsets)
    }
}
