use crate::{
    constants::cs2,
    cs2::{CS2, schema::Schema},
};

macro_rules! field_stmt {
    ($proc:ident, $off:ident, $client:ident, $physics:ident, $field:ident, $fdef:ident, lib ($lib:ident)) => {{
        $off.$field.$fdef = $proc.module_base_address(cs2::$lib)?;
    }};
    ($proc:ident, $off:ident, $client:ident, $physics:ident, $field:ident, $fdef:ident, interface ($lib:ident, $name:literal)) => {{
        let Some($fdef) = $proc.get_interface_offset($off.library.$lib, $name) else {
            utils::warn!(concat!(
                "could not find '",
                stringify!($fdef),
                "' interface offset"
            ));
            return None;
        };
        $off.$field.$fdef = $fdef;
    }};
    ($proc:ident, $off:ident, $client:ident, $physics:ident, $field:ident, $fdef:ident, convar ($name:literal)) => {{
        let Some($fdef) = $proc.get_convar($off.interface.cvar, $name) else {
            utils::warn!(concat!(
                "could not find '",
                stringify!($fdef),
                "' convar offset"
            ));
            return None;
        };
        $off.$field.$fdef = $fdef;
    }};
    ($proc:ident, $off:ident, $client:ident, $physics:ident, $field:ident, $fdef:ident, scan ($sig:literal, $lib:ident, rel($a:literal, $b:literal))) => {{
        let Some($fdef) = $proc.scan($sig, $off.library.$lib) else {
            utils::warn!(concat!("could not find '", stringify!($fdef), "' offset"));
            return None;
        };
        $off.$field.$fdef = $proc.get_relative_address($fdef, $a, $b);
    }};
    ($proc:ident, $off:ident, $client:ident, $physics:ident, $field:ident, $fdef:ident, scan ($sig:literal, $lib:ident, rel($a:literal, $b:literal, $c:literal))) => {{
        let Some($fdef) = $proc.scan($sig, $off.library.$lib) else {
            utils::warn!(concat!("could not find '", stringify!($fdef), "' offset"));
            return None;
        };
        $off.$field.$fdef = $proc.get_relative_address($fdef + $a, $b, $c);
    }};
    ($proc:ident, $off:ident, $client:ident, $physics:ident, $field:ident, $fdef:ident, scan_ptr ($sig:literal, $lib:ident, rel($a:literal, $b:literal))) => {{
        let Some($fdef) = $proc.scan($sig, $off.library.$lib) else {
            utils::warn!(concat!("could not find '", stringify!($fdef), "' offset"));
            return None;
        };
        let $fdef = $proc.get_relative_address($fdef, $a, $b);
        $off.$field.$fdef = $proc.read($fdef);
    }};
    ($proc:ident, $off:ident, $client:ident, $physics:ident, $field:ident, $fdef:ident, class_size ($class:literal)) => {{
        $off.$field.$fdef = $client.get_class($class)?.size() as _;
    }};
    ($proc:ident, $off:ident, $client:ident, $physics:ident, $field:ident, $fdef:ident, fixed ($value:literal)) => {{
        $off.$field.$fdef = $value;
    }};
    ($proc:ident, $off:ident, $client:ident, $physics:ident, $field:ident, $fdef:ident, schema ($class:literal, $field_name:literal)) => {{
        $off.$field.$fdef = $client.get($class, $field_name)?;
    }};
    ($proc:ident, $off:ident, $client:ident, $physics:ident, $field:ident, $fdef:ident, phys ($class:literal, $field_name:literal)) => {{
        $off.$field.$fdef = $physics.get($class, $field_name)?;
    }};
    ($proc:ident, $off:ident, $client:ident, $physics:ident, $field:ident, $fdef:ident, schema_sub ($class:literal, $field_name:literal, $sub:literal)) => {{
        $off.$field.$fdef = $client.get($class, $field_name)? - $sub;
    }};
    ($proc:ident, $off:ident, $client:ident, $physics:ident, $field:ident, $fdef:ident, deref ($src:ident, $off_lit:literal, $add_lit:literal)) => {{
        $off.$field.$fdef = $proc.read::<usize>($off.$field.$src + $off_lit) + $add_lit;
    }};
    ($proc:ident, $off:ident, $client:ident, $physics:ident, $field:ident, $fdef:ident, interface_fn ($grp:ident, $src:ident, $idx:literal, $off_lit:literal)) => {{
        $off.$field.$fdef = $proc
            .read::<u32>($proc.get_interface_function($off.$grp.$src, $idx) + $off_lit)
            as usize;
    }};
    ($proc:ident, $off:ident, $client:ident, $physics:ident, $field:ident, $fdef:ident, module_export ($lib:ident, $name:literal, rel($a:literal, $b:literal, $c:literal, $d:literal))) => {{
        let Some(window) = $proc.get_module_export($off.library.$lib, $name) else {
            utils::warn!(concat!("could not find '", stringify!($fdef), "' offset"));
            return None;
        };
        let window = $proc.get_relative_address(window, $a, $b);
        let window = $proc.read(window);
        $off.$field.$fdef = $proc.get_relative_address(window, $c, $d);
    }};
}

macro_rules! schema {
    (@struct $field:ident : $name:ident { $($def:tt)* } $($rest:tt)*) => {
        schema!(@sdef $name $($def)*);
        schema!(@struct $($rest)*);
    };
    (@struct $scope:ident : { $($inner:tt)* } $($rest:tt)*) => {
        schema!(@struct $($inner)*);
        schema!(@struct $($rest)*);
    };
    (@struct) => {};

    (@sdef $name:ident $($fdef:ident : $kind:ident $args:tt ,)*) => {
        #[derive(Default)]
        pub struct $name {
            $(pub $fdef: usize,)*
        }
    };

    (@offsets_acc { $($fields:tt)* } $field:ident : $name:ident { $($def:tt)* } $($rest:tt)*) => {
        schema!(@offsets_acc { $($fields)* pub $field: $name, } $($rest)*);
    };
    (@offsets_acc { $($fields:tt)* } $scope:ident : { $($inner:tt)* } $($rest:tt)*) => {
        schema!(@offsets_acc { $($fields)* } $($inner)* $($rest)*);
    };
    (@offsets_acc { $($fields:tt)* }) => {
        #[derive(Default)]
        pub struct Offsets {
            $($fields)*
        }
    };

    (@fn $proc:ident $off:ident $schema:ident $client:ident $physics:ident $field:ident : $name:ident { $($fdef:ident : $kind:ident $args:tt ,)* } $($rest:tt)+) => {
        $(field_stmt!($proc, $off, $client, $physics, $field, $fdef, $kind $args);)*
        schema!(@fn $proc $off $schema $client $physics $($rest)*);
    };
    (@fn $proc:ident $off:ident $schema:ident $client:ident $physics:ident $field:ident : $name:ident { $($fdef:ident : $kind:ident $args:tt ,)* }) => {
        $(field_stmt!($proc, $off, $client, $physics, $field, $fdef, $kind $args);)*
    };
    (@fn $proc:ident $off:ident $schema:ident $client:ident $physics:ident client: { $($inner:tt)* } $($rest:tt)+) => {
        let $schema = Schema::new($proc, $off.library.schema)?;
        let $client = $schema.get_library(cs2::CLIENT_LIB)?;
        schema!(@fn $proc $off $schema $client $physics $($inner)* $($rest)*);
    };
    (@fn $proc:ident $off:ident $schema:ident $client:ident $physics:ident client: { $($inner:tt)* }) => {
        let $schema = Schema::new($proc, $off.library.schema)?;
        let $client = $schema.get_library(cs2::CLIENT_LIB)?;
        schema!(@fn $proc $off $schema $client $physics $($inner)*)
    };
    (@fn $proc:ident $off:ident $schema:ident $client:ident $physics:ident physics: { $($inner:tt)* } $($rest:tt)+) => {
        let $physics = $schema.get_library(cs2::PHYSICS_LIB)?;
        schema!(@fn $proc $off $schema $client $physics $($inner)* $($rest)*);
    };
    (@fn $proc:ident $off:ident $schema:ident $client:ident $physics:ident physics: { $($inner:tt)* }) => {
        let $physics = $schema.get_library(cs2::PHYSICS_LIB)?;
        schema!(@fn $proc $off $schema $client $physics $($inner)*)
    };
    (@fn $proc:ident $off:ident $schema:ident $client:ident $physics:ident) => {};

    ($($group:tt)*) => {
        schema!(@struct $($group)*);
        schema!(@offsets_acc {} $($group)*);
        macro_rules! find_offsets_body {
            ($proc:ident, $off:ident, $schema:ident, $client:ident, $physics:ident) => {
                schema!(@fn $proc $off $schema $client $physics $($group)*)
            };
        }
    };
}

schema! {
    library: LibraryOffsets {
        client: lib(CLIENT_LIB),
        engine: lib(ENGINE_LIB),
        tier0: lib(TIER0_LIB),
        input: lib(INPUT_LIB),
        sdl: lib(SDL_LIB),
        schema: lib(SCHEMA_LIB),
        physics: lib(PHYSICS_LIB),
    }

    interface: InterfaceOffsets {
        resource: interface(engine, "GameResourceServiceClientV0"),
        entity: deref(resource, 0x50, 0x10),
        cvar: interface(tier0, "VEngineCvar0"),
        input: interface(input, "InputSystemVersion0"),
    }

    direct: DirectOffsets {
        local_player: scan("48 83 3D ? ? ? ? 00 0F 95 C0 C3", client, rel(0x03, 0x08)),
        button_state: interface_fn(interface, input, 19, 0x14),
        view_matrix: scan("C6 83 ? ? 00 00 01 4C 8D 05", client, rel(0x0A, 0x00, 0x04)),
        sdl_window: module_export(sdl, "SDL_GetKeyboardFocus", rel(0x02, 0x06, 0x03, 0x07)),
        global_vars: scan("48 8D 05 ? ? ? ? 45 31 E4 48 8B 00 8B 78 10", client, rel(0x03, 0x07)),
        vphys_world: scan_ptr("4c 8d 35 ? ? ? ? 49 8b 3e e8 ? ? ? ? 48 89 c2", client, rel(3, 7)),
        build_date: scan("4c 89 e6 e8 ? ? ? ? 48 8d 35 ? ? ? ? 48 8d 3d", engine, rel(11, 15)),
    }

    convar: ConvarOffsets {
        ffa: convar("mp_teammates_are_enemies"),
        sensitivity: convar("sensitivity"),
    }

    client: {
        controller: PlayerControllerOffsets {
            steam_id: schema("CBasePlayerController", "m_steamID"),
            money_services: schema("CCSPlayerController", "m_pInGameMoneyServices"),
            money: schema(
                "CCSPlayerController_InGameMoneyServices",
                "m_iAccount"
            ),
            color: schema("CCSPlayerController", "m_iCompTeammateColor"),
            name: schema("CBasePlayerController", "m_iszPlayerName"),
            pawn: schema("CBasePlayerController", "m_hPawn"),
            desired_fov: schema("CBasePlayerController", "m_iDesiredFOV"),
            owner_entity: schema("C_BaseEntity", "m_hOwnerEntity"),
            rank: schema("CCSPlayerController", "m_iCompetitiveRanking"),
            rank_type: schema("CCSPlayerController", "m_iCompetitiveRankType"),
            action_tracking_services: schema("CCSPlayerController", "m_pActionTrackingServices"),
        }
        entity: EntityOffsets {
            health: schema("C_BaseEntity", "m_iHealth"),
            max_health: schema("C_BaseEntity", "m_iMaxHealth"),
            team: schema("C_BaseEntity", "m_iTeamNum"),
            life_state: schema("C_BaseEntity", "m_lifeState"),
            game_scene_node: schema("C_BaseEntity", "m_pGameSceneNode"),
            velocity: schema("C_BaseEntity", "m_vecVelocity"),
            collision: schema("C_BaseEntity", "m_pCollision"),
        }
        collision: CollisionOffsets {
            mins: schema("CCollisionProperty", "m_vecMins"),
            maxs: schema("CCollisionProperty", "m_vecMaxs"),
        }
        pawn: PawnOffsets {
            controller: schema("C_BasePlayerPawn", "m_hController"),
            armor: schema("C_CSPlayerPawn", "m_ArmorValue"),
            fov_multiplier: schema("C_BasePlayerPawn", "m_flFOVSensitivityAdjust"),
            eye_offset: schema("C_BaseModelEntity", "m_vecViewOffset"),
            shots_fired: schema("C_CSPlayerPawn", "m_iShotsFired"),
            view_angles: schema("C_BasePlayerPawn", "v_angle"),
            eye_angles: schema("C_CSPlayerPawn", "m_angEyeAngles"),
            flags: schema("C_BaseEntity", "m_fFlags"),
            crosshair_entity: schema("C_CSPlayerPawn", "m_iIDEntIndex"),
            is_scoped: schema("C_CSPlayerPawn", "m_bIsScoped"),
            deathmatch_immunity: schema("C_CSPlayerPawn", "m_bGunGameImmunity"),
            observer_services: schema("C_BasePlayerPawn", "m_pObserverServices"),
            spotted_state: schema("C_CSPlayerPawn", "m_entitySpottedState"),
            flash_alpha: schema("C_CSPlayerPawnBase", "m_flFlashMaxAlpha"),
            flash_duration: schema("C_CSPlayerPawnBase", "m_flFlashDuration"),
            camera_services: schema("C_BasePlayerPawn", "m_pCameraServices"),
            item_services: schema("C_BasePlayerPawn", "m_pItemServices"),
            weapon_services: schema("C_BasePlayerPawn", "m_pWeaponServices"),
            aim_punch_services: schema("C_CSPlayerPawn", "m_pAimPunchServices"),
            bullet_services: schema("C_CSPlayerPawn", "m_pBulletServices"),
        }
        game_scene_node: GameSceneNodeOffsets {
            dormant: schema("CGameSceneNode", "m_bDormant"),
            origin: schema("CGameSceneNode", "m_vecAbsOrigin"),
            node_to_world: schema("CGameSceneNode", "m_nodeToWorld"),
            model_state: schema("CSkeletonInstance", "m_modelState"),
        }
        smoke: SmokeOffsets {
            did_smoke_effect: schema("C_SmokeGrenadeProjectile", "m_bDidSmokeEffect"),
            smoke_color: schema("C_SmokeGrenadeProjectile", "m_vSmokeColor"),
        }
        molotov: MolotovOffsets {
            is_incendiary: schema("C_MolotovProjectile", "m_bIsIncGrenade"),
        }
        inferno: InfernoOffsets {
            is_burning: schema("C_Inferno", "m_bFireIsBurning"),
            fire_count: schema("C_Inferno", "m_fireCount"),
            fire_positions: schema("C_Inferno", "m_firePositions"),
        }
        spotted_state: SpottedStateOffsets {
            spotted: schema("EntitySpottedState_t", "m_bSpotted"),
            mask: schema("EntitySpottedState_t", "m_bSpottedByMask"),
        }
        camera_services: CameraServicesOffsets {
            fov: schema("CCSPlayerBase_CameraServices", "m_iFOV"),
        }
        item_services: ItemServicesOffsets {
            has_defuser: schema("CCSPlayer_ItemServices", "m_bHasDefuser"),
            has_helmet: schema("CCSPlayer_ItemServices", "m_bHasHelmet"),
        }
        weapon_services: WeaponServicesOffsets {
            weapons: schema("CPlayer_WeaponServices", "m_hMyWeapons"),
            active_weapon: schema("CPlayer_WeaponServices", "m_hActiveWeapon"),
        }
        aim_punch_services: AimPunchServicesOffsets {
            aim_punch_cache: schema_sub("CCSPlayer_AimPunchServices", "m_unpredictableBaseTick", 0x18),
        }
        action_tracking: ActionTrackingServicesOffsets {
            round_kills: schema("CCSPlayerController_ActionTrackingServices", "m_iNumRoundKills"),
            round_damage: schema("CCSPlayerController_ActionTrackingServices", "m_flTotalRoundDamageDealt"),
            per_round_stats: schema("CCSPlayerController_ActionTrackingServices", "m_perRoundStats"),
        }
        bullet_services: BulletServicesOffsets {
            total_hits: schema("CCSPlayer_BulletServices", "m_totalHitsOnServer"),
        }
        per_round_stats: PerRoundStatsOffsets {
            kills: schema("CSPerRoundStats_t", "m_iKills"),
            deaths: schema("CSPerRoundStats_t", "m_iDeaths"),
            assists: schema("CSPerRoundStats_t", "m_iAssists"),
            damage: schema("CSPerRoundStats_t", "m_iDamage"),
            size: class_size("CSPerRoundStats_t"),
        }
        observer_services: ObserverServicesOffsets {
            target: schema("CPlayer_ObserverServices", "m_hObserverTarget"),
        }
        econ_item_view: EconItemViewOffsets {
            item_definition_index: schema("C_EconItemView", "m_iItemDefinitionIndex"),
        }
        weapon: WeaponOffsets {
            attribute_manager: schema("C_EconEntity", "m_AttributeManager"),
            item: schema("C_AttributeContainer", "m_Item"),
            item_definition_index: schema("C_EconItemView", "m_iItemDefinitionIndex"),
            clip_primary: schema("C_BasePlayerWeapon", "m_iClip1"),
            reserve_ammo: schema("C_BasePlayerWeapon", "m_pReserveAmmo"),
        }
        planted_c4: PlantedC4Offsets {
            is_ticking: schema("C_PlantedC4", "m_bBombTicking"),
            blow_time: schema("C_PlantedC4", "m_flC4Blow"),
            being_defused: schema("C_PlantedC4", "m_bBeingDefused"),
            is_defused: schema("C_PlantedC4", "m_bBombDefused"),
            has_exploded: schema("C_PlantedC4", "m_bHasExploded"),
            defuse_time: schema("C_PlantedC4", "m_flDefuseCountDown"),
            defuse_time_left: schema("C_PlantedC4", "m_flDefuseCountDown"),
        }
        model_state: ModelState {
            skeleton_instance: schema("CBodyComponentSkeletonInstance", "m_skeletonInstance"),
        }
        network_velocity: NetworkVelocityOffsets {
            x: fixed(0x10),
            y: fixed(0x18),
            z: fixed(0x20),
        }
        entity_identity: EntityIdentityOffsets {
            size: class_size("CEntityIdentity"),
        }
    }

    physics: {
        hull: PhysHullOffsets {
            vertices: phys("RnHull_t", "m_VertexPositions"),
            edges: phys("RnHull_t", "m_Edges"),
            faces: phys("RnHull_t", "m_Faces"),
            flags: phys("RnHull_t", "m_nFlags"),
        }
        mesh: PhysMeshOffsets {
            vertices: phys("RnMesh_t", "m_Vertices"),
            triangles: phys("RnMesh_t", "m_Triangles"),
            materials: phys("RnMesh_t", "m_Materials"),
            flags: phys("RnMesh_t", "m_nFlags"),
        }
    }
}

impl CS2 {
    pub fn find_offsets(&self) -> Option<Offsets> {
        let proc = &self.process;
        let mut offsets = Offsets::default();
        find_offsets_body!(proc, offsets, schema, client, physics);
        Some(offsets)
    }
}
