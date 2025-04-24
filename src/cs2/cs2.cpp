#include "cs2/cs2.hpp"

#include <mithril/hex.hpp>
#include <mithril/logging.hpp>
#include <optional>
#include <string>
#include <thread>

#include "config.hpp"
#include "cs2/constants.hpp"
#include "cs2/features.hpp"
#include "cs2/player.hpp"
#include "math.hpp"
#include "process.hpp"

bool is_valid {false};
Process process {};
Offsets offsets {};
Target target {};
glm::vec2 aim_punch {};
std::vector<Player> players {};

void CS2() {
    logging::Info("game thread started");
    while (true) {
        const auto clock = std::chrono::steady_clock::now();

        if (flags.should_quit) {
            break;
        }

        if (IsValid()) {
            config_lock.lock();
            Run();
            config_lock.unlock();
        } else {
            Setup();
        }

        const auto end_time = std::chrono::steady_clock::now();
        const auto us = std::chrono::duration_cast<std::chrono::microseconds>(end_time - clock);
        const auto frame_time = std::chrono::microseconds(10000);
        if (IsValid()) {
            if (us > frame_time * 2) {
                logging::Debug(
                    "aimbot thread took {} ms, max is {} ms", us.count() / 1000,
                    frame_time.count() / 1000);
            }
            std::this_thread::sleep_for(frame_time - us);
        } else {
            // if it was just a 5 second sleep, it would wait 5 seconds before closing the gui
            // window
            for (i32 i = 0; i < 100; i++) {
                config_lock.lock();
                if (flags.should_quit) {
                    return;
                }
                config_lock.unlock();
                std::this_thread::sleep_for(std::chrono::milliseconds(50));
            }
        }
    }
}

bool IsValid() {
    if (!process.pid) {
        return false;
    }
    return is_valid && ValidatePid(process.pid);
}

void Setup() {
    const std::optional<i32> pid = GetPid(PROCESS_NAME);
    if (!pid) {
        logging::Debug("game not running");
        is_valid = false;
        return;
    }

    const std::optional<Process> new_process = OpenProcess(*pid);
    if (!new_process) {
        logging::Debug("could not open process");
        is_valid = false;
        return;
    }
    process = *new_process;
    logging::Info("game started, pid: {}", *pid);

    const std::optional<Offsets> new_offsets = FindOffsets();
    if (!new_offsets) {
        is_valid = false;
        return;
    }
    offsets = *new_offsets;

    is_valid = true;
}

std::optional<Offsets> FindOffsets() {
    Offsets offsets = {};

    // get library base addresses, will fail if game is not yet fully loaded
    const std::optional<u64> client_address = process.GetModuleBaseAddress(CLIENT_LIB);
    if (!client_address) {
        return std::nullopt;
    }
    logging::Debug("{} at: {}", CLIENT_LIB, hex::HexString(*client_address));

    const std::optional<u64> engine_address = process.GetModuleBaseAddress(ENGINE_LIB);
    if (!engine_address) {
        return std::nullopt;
    }
    logging::Debug("{} at: {}", ENGINE_LIB, hex::HexString(*engine_address));

    const std::optional<u64> tier0_address = process.GetModuleBaseAddress(TIER0_LIB);
    if (!tier0_address) {
        return std::nullopt;
    }
    logging::Debug("{} at: {}", TIER0_LIB, hex::HexString(*tier0_address));

    const std::optional<u64> input_address = process.GetModuleBaseAddress(INPUT_LIB);
    if (!input_address) {
        return std::nullopt;
    }
    logging::Debug("{} at: {}", INPUT_LIB, hex::HexString(*input_address));

    const std::optional<u64> sdl_address = process.GetModuleBaseAddress(SDL_LIB);
    if (!sdl_address) {
        return std::nullopt;
    }
    logging::Debug("{} at: {}", SDL_LIB, hex::HexString(*sdl_address));

    const std::optional<u64> matchmaking_address = process.GetModuleBaseAddress(MATCH_LIB);
    if (!matchmaking_address) {
        return std::nullopt;
    }
    logging::Debug("{} at: {}", MATCH_LIB, hex::HexString(*matchmaking_address));

    offsets.library.client = *client_address;
    offsets.library.engine = *engine_address;
    offsets.library.tier0 = *tier0_address;
    offsets.library.input = *input_address;
    offsets.library.sdl = *sdl_address;
    offsets.library.matchmaking = *matchmaking_address;

    // used for player interface offset
    const std::optional<u64> resource_offset =
        process.GetInterfaceOffset(offsets.library.engine, "GameResourceServiceClientV0");
    if (!resource_offset) {
        logging::Error("failed to get resource offset");
        return std::nullopt;
    }
    offsets.interface.resource = *resource_offset;
    logging::Debug("resource interface offset at: {}", hex::HexString(offsets.interface.resource));
    offsets.interface.entity = process.Read<u64>(offsets.interface.resource + 0x50);
    logging::Debug("entity list offset at: {}", hex::HexString(offsets.interface.entity));
    offsets.interface.player = offsets.interface.entity + 0x10;
    logging::Debug(
        "local player interface offset at: {}", hex::HexString(offsets.interface.player));

    const std::optional<u64> local_player = process.ScanPattern(
        {0x48, 0x83, 0x3D, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0F, 0x95, 0xC0, 0xC3},
        {true, true, true, false, false, false, false, true, true, true, true, true}, 12,
        offsets.library.client);
    if (!local_player) {
        logging::Error("failed to get local player offset");
        return std::nullopt;
    }
    offsets.direct.local_player = process.GetRelativeAddress(*local_player, 0x03, 0x08);
    logging::Debug("local player offset at: {}", hex::HexString(offsets.direct.local_player));

    const std::optional<u64> cvar_address =
        process.GetInterfaceOffset(offsets.library.tier0, "VEngineCvar0");
    if (!cvar_address) {
        logging::Error("failed to get cvar offset");
        return std::nullopt;
    }
    offsets.interface.cvar = *cvar_address;
    logging::Debug("convar offset at: {}", hex::HexString(offsets.interface.cvar));

    const std::optional<u64> input_system_address =
        process.GetInterfaceOffset(offsets.library.input, "InputSystemVersion0");
    if (!input_system_address) {
        logging::Error("failed to get input offset");
        return std::nullopt;
    }
    offsets.interface.input = *input_system_address;
    logging::Debug("input interface offset at: {}", hex::HexString(offsets.interface.input));

    const std::optional<u64> view_matrix = process.ScanPattern(
        {0x48, 0x8D, 0x05, 0x00, 0x00, 0x00, 0x00, 0x4C, 0x8D, 0x05, 0x00, 0x00, 0x00, 0x00, 0x48,
         0x8D, 0x0D},
        {
            true,
            true,
            true,
            false,
            false,
            false,
            false,
            true,
            true,
            true,
            false,
            false,
            false,
            false,
            true,
            true,
            true,
        },
        17, offsets.library.client);
    if (!view_matrix) {
        logging::Error("could not find view matrix offset");
        return std::nullopt;
    }
    offsets.direct.view_matrix = process.GetRelativeAddress(*view_matrix + 0x07, 0x03, 0x07);
    logging::Debug("view matrix offset at: {}", hex::HexString(offsets.direct.view_matrix));

    offsets.direct.button_state =
        process.Read<u32>(process.GetInterfaceFunction(offsets.interface.input, 19) + 0x14);
    logging::Debug("button state offset at: {}", hex::HexString(offsets.direct.button_state));

    const std::optional<u64> game_types = process.ScanPattern(
        {0x48, 0x8D, 0x05, 0x00, 0x00, 0x00, 0x00, 0xC3, 0x0F, 0x1F, 0x84, 0x00, 0x00, 0x00, 0x00,
         0x00, 0x48, 0x8B, 0x07},
        {
            true, true,  true,  false, false, false, false, true, true, true,
            true, false, false, false, false, false, true,  true, true,
        },
        19, offsets.library.matchmaking);
    if (!game_types) {
        logging::Error("could not find map name offset");
        return std::nullopt;
    }
    offsets.direct.game_types = process.GetRelativeAddress(*game_types, 0x03, 0x07);
    logging::Debug("map name offset at: {}", hex::HexString(offsets.direct.game_types));

    const std::optional<u64> planted_c4 = process.ScanPattern(
        {0x00, 0x00, 0x00, 0x00, 0x8B, 0x10, 0x85, 0xD2, 0x0F, 0x8F},
        {false, false, false, false, true, true, true, true, true, true}, 10,
        offsets.library.client);
    if (!planted_c4) {
        logging::Error("could not find planted c4 offset");
        // todo: verify this works
        // return std::nullopt;
    }
    offsets.direct.planted_c4 = process.GetRelativeAddress(*planted_c4, 0x00, 0x07);
    logging::Debug("planted c4 offset at: {}", hex::HexString(offsets.direct.planted_c4));

    const std::optional<u64> sdl_window_address =
        process.GetModuleExport(offsets.library.sdl, "SDL_GetKeyboardFocus");
    if (!sdl_window_address) {
        logging::Error("could not find sdl window offset");
    }
    const u64 sdl_window = process.GetRelativeAddress(*sdl_window_address, 0x02, 0x06);
    const u64 sdl_window2 = process.Read<u64>(sdl_window);
    offsets.direct.sdl_window = process.GetRelativeAddress(sdl_window2, 0x03, 0x07);
    logging::Debug("sdl window offset at: {}", hex::HexString(offsets.direct.sdl_window));

    const std::optional<u64> ffa_address =
        process.GetConvar(offsets.interface.cvar, "mp_teammates_are_enemies");
    if (!ffa_address) {
        logging::Error("could not get mp_tammates_are_enemies convar offset");
    }
    offsets.convar.ffa = *ffa_address;
    logging::Debug("ffa convar offset at: {}", hex::HexString(offsets.convar.ffa));

    const std::optional<u64> sensitivity_address =
        process.GetConvar(offsets.interface.cvar, "sensitivity");
    if (!sensitivity_address) {
        logging::Error("could not get sensitivity convar offset");
    }
    offsets.convar.sensitivity = *sensitivity_address;
    logging::Debug("sensitivity convar offset at: {}", hex::HexString(offsets.convar.sensitivity));

    // dump all netvars from client lib
    const u64 base = offsets.library.client;
    const u64 size = process.ModuleSize(base);

    const std::vector<u8> client_dump_vec = process.DumpModule(base);
    const u8 *client_dump = client_dump_vec.data();

    for (size_t i = size - 8; i > 0; i -= 8) {
        // read client dump at i from dump directly
        const u64 entry = reinterpret_cast<u64>(client_dump) + i;

        bool network_enable = false;
        const u64 network_enable_pointer = *reinterpret_cast<u64 *>(entry);

        if (network_enable_pointer == 0) {
            continue;
        }

        if (network_enable_pointer >= base && network_enable_pointer <= base + size) {
            const u64 network_enable_name_pointer =
                *reinterpret_cast<const u64 *>(network_enable_pointer - base + client_dump);
            if (network_enable_name_pointer >= base && network_enable_name_pointer <= base + size) {
                const std::string name {reinterpret_cast<const char *>(
                    network_enable_name_pointer - base + client_dump)};
                if (name == "MNetworkEnable") {
                    network_enable = true;
                }
            }
        }

        u64 name_pointer = 0;
        if (network_enable == false) {
            name_pointer = *reinterpret_cast<u64 *>(entry);
        } else {
            name_pointer = *reinterpret_cast<u64 *>(entry + 0x08);
        }

        if (name_pointer < base || name_pointer > base + size) {
            continue;
        }

        const auto name =
            std::string(reinterpret_cast<const char *>(name_pointer - base + client_dump));

        if (name == "m_sSanitizedPlayerName") {
            if (!network_enable || offsets.controller.name != 0) {
                continue;
            }
            offsets.controller.name = *reinterpret_cast<i32 *>(entry + 0x18);
            logging::Debug(
                "player name netvar offset: {}", hex::HexString(offsets.controller.name));
        } else if (name == "m_hPawn") {
            if (!network_enable || offsets.controller.pawn != 0) {
                continue;
            }
            offsets.controller.pawn = *reinterpret_cast<i32 *>(entry + 0x18);
            logging::Debug(
                "player pawn netvar offset: {}", hex::HexString(offsets.controller.pawn));
        } else if (name == "m_steamID") {
            if (!network_enable || offsets.controller.steam_id != 0) {
                continue;
            }
            offsets.controller.steam_id = *reinterpret_cast<i32 *>(entry + 0x18);
            logging::Debug(
                "player steam id netvar offset: {}", hex::HexString(offsets.controller.steam_id));
        } else if (name == "m_iDesiredFOV") {
            if (offsets.controller.desired_fov != 0) {
                continue;
            }
            offsets.controller.desired_fov = *reinterpret_cast<i32 *>(entry + 0x08);
            logging::Debug(
                "player desired fov netvar offset: {}",
                hex::HexString(offsets.controller.desired_fov));
        } else if (name == "m_hOwnerEntity") {
            if (!network_enable || offsets.controller.owner_entity != 0) {
                continue;
            }
            offsets.controller.owner_entity = *reinterpret_cast<i32 *>(entry + 0x18);
            logging::Debug(
                "entity owner netvar offset: {}", hex::HexString(offsets.controller.owner_entity));
        } else if (name == "m_iHealth") {
            if (!network_enable || offsets.pawn.health != 0) {
                continue;
            }
            offsets.pawn.health = *reinterpret_cast<i32 *>(entry + 0x18);
            logging::Debug(

                "player health netvar offset: {}", hex::HexString(offsets.pawn.health));
        } else if (name == "m_ArmorValue") {
            if (!network_enable || offsets.pawn.armor != 0) {
                continue;
            }
            offsets.pawn.armor = *reinterpret_cast<i32 *>(entry + 0x18);
        } else if (name == "m_iTeamNum") {
            if (!network_enable || offsets.pawn.team != 0) {
                continue;
            }
            offsets.pawn.team = *reinterpret_cast<i32 *>(entry + 0x18);
            logging::Debug("player team netvar offset: {}", hex::HexString(offsets.pawn.team));
        } else if (name == "m_lifeState") {
            if (!network_enable || offsets.pawn.life_state != 0) {
                continue;
            }
            offsets.pawn.life_state = *reinterpret_cast<i32 *>(entry + 0x18);
            logging::Debug(

                "player life state netvar offset: {}", hex::HexString(offsets.pawn.life_state));
        } else if (name == "m_pClippingWeapon") {
            if (offsets.pawn.weapon != 0) {
                continue;
            }
            offsets.pawn.weapon = *reinterpret_cast<i32 *>(entry + 0x10);
            logging::Debug(

                "player weapon netvar offset: {}", hex::HexString(offsets.pawn.weapon));
        } else if (name == "m_flFOVSensitivityAdjust") {
            if (offsets.pawn.fov_multiplier != 0) {
                continue;
            }
            offsets.pawn.fov_multiplier = *reinterpret_cast<i32 *>(entry + 0x08);
            logging::Debug(
                "player fov multiplier netvar offset: {}",
                hex::HexString(offsets.pawn.fov_multiplier));
        } else if (name == "m_pGameSceneNode") {
            if (offsets.pawn.game_scene_node != 0) {
                continue;
            }
            offsets.pawn.game_scene_node = *reinterpret_cast<i32 *>(entry + 0x10);
            logging::Debug(
                "player game scene node netvar offset: {}",
                hex::HexString(offsets.pawn.game_scene_node));
        } else if (name == "m_vecViewOffset") {
            if (!network_enable || offsets.pawn.eye_offset != 0) {
                continue;
            }
            offsets.pawn.eye_offset = *reinterpret_cast<i32 *>(entry + 0x18);
            logging::Debug(
                "player eye offset netvar offset: {}", hex::HexString(offsets.pawn.eye_offset));
        } else if (name == "m_aimPunchCache") {
            if (!network_enable || offsets.pawn.aim_punch_cache != 0) {
                continue;
            }
            offsets.pawn.aim_punch_cache = *reinterpret_cast<i32 *>(entry + 0x18);
            logging::Debug(
                "player aim punch netvar offset: {}", hex::HexString(offsets.pawn.aim_punch_cache));
        } else if (name == "m_iShotsFired") {
            if (!network_enable || offsets.pawn.shots_fired != 0) {
                continue;
            }
            offsets.pawn.shots_fired = *reinterpret_cast<i32 *>(entry + 0x18);
            logging::Debug(
                "player shots fired netvar offset: {}", hex::HexString(offsets.pawn.shots_fired));
        } else if (name == "v_angle") {
            if (offsets.pawn.view_angles != 0) {
                continue;
            }
            offsets.pawn.view_angles = *reinterpret_cast<i32 *>(entry + 0x08);
            logging::Debug(
                "player view angle netvar offset: {}", hex::HexString(offsets.pawn.view_angles));
        } else if (name == "m_angEyeAngles") {
            if (offsets.pawn.eye_angles != 0) {
                continue;
            }
            offsets.pawn.eye_angles = *reinterpret_cast<i32 *>(entry + 0x10);
            logging::Debug(
                "player eye angle netvar offset: {}", hex::HexString(offsets.pawn.eye_angles));
        } else if (name == "m_flFlashMaxAlpha") {
            if (offsets.pawn.flash_alpha != 0) {
                continue;
            }
            offsets.pawn.flash_alpha = *reinterpret_cast<i32 *>(entry + 0x10);
            logging::Debug(
                "player flash alpha netvar offset: {}", hex::HexString(offsets.pawn.flash_alpha));
        } else if (name == "m_flFlashDuration") {
            if (offsets.pawn.flash_duration != 0) {
                continue;
            }
            offsets.pawn.flash_duration = *reinterpret_cast<i32 *>(entry + 0x10);
            logging::Debug(
                "player flash duration netvar offset: {}",
                hex::HexString(offsets.pawn.flash_duration));
        } else if (name == "m_bIsScoped") {
            if (!network_enable || offsets.pawn.scoped != 0) {
                continue;
            }
            offsets.pawn.scoped = *reinterpret_cast<i32 *>(entry + 0x18);
            logging::Debug("player scoped netvar offset: {}", hex::HexString(offsets.pawn.scoped));
        } else if (name == "m_entitySpottedState") {
            if (!network_enable || offsets.pawn.spotted_state != 0) {
                continue;
            }
            const u64 offset = *reinterpret_cast<i32 *>(entry + 0x18);
            if (offset < 10000 || offset > 14000) {
                continue;
            }
            offsets.pawn.spotted_state = offset;
            logging::Debug(
                "player spotted state netvar offset: {}",
                hex::HexString(offsets.pawn.spotted_state));
        } else if (name == "m_iIDEntIndex") {
            if (offsets.pawn.crosshair_entity != 0) {
                continue;
            }
            offsets.pawn.crosshair_entity = *reinterpret_cast<i32 *>(entry + 0x10);
            logging::Debug(
                "player crosshair entity netvar offset: {}",
                hex::HexString(offsets.pawn.crosshair_entity));
        } else if (name == "m_pObserverServices") {
            if (offsets.pawn.observer_services != 0) {
                continue;
            }
            offsets.pawn.observer_services = *reinterpret_cast<i32 *>(entry + 0x08);
            logging::Debug(
                "player observer service netvar offset: {}",
                hex::HexString(offsets.pawn.observer_services));
        } else if (name == "m_pCameraServices") {
            if (!network_enable || offsets.pawn.camera_services != 0) {
                continue;
            }
            offsets.pawn.camera_services = *reinterpret_cast<i32 *>(entry + 0x18);
            logging::Debug(
                "player camera service netvar offset: {}",
                hex::HexString(offsets.pawn.camera_services));
        } else if (name == "m_pItemServices") {
            if (offsets.pawn.item_services != 0) {
                continue;
            }
            offsets.pawn.item_services = *reinterpret_cast<i32 *>(entry + 0x08);
            logging::Debug(
                "player item service netvar offset: {}",
                hex::HexString(offsets.pawn.item_services));
        } else if (name == "m_pWeaponServices") {
            if (offsets.pawn.weapon_services != 0) {
                continue;
            }
            offsets.pawn.weapon_services = *reinterpret_cast<i32 *>(entry + 0x08);
            logging::Debug(
                "player weapon service netvar offset: {}",
                hex::HexString(offsets.pawn.weapon_services));
        } else if (name == "m_bDormant") {
            if (offsets.game_scene_node.dormant != 0) {
                continue;
            }
            offsets.game_scene_node.dormant = *reinterpret_cast<i32 *>(entry + 0x08);
            logging::Debug(
                "player dormant netvar offset: {}",
                hex::HexString(offsets.game_scene_node.dormant));
        } else if (name == "m_vecAbsOrigin") {
            if (!network_enable || offsets.game_scene_node.origin != 0) {
                continue;
            }
            offsets.game_scene_node.origin = *reinterpret_cast<i32 *>(entry + 0x18);
            logging::Debug(
                "player position netvar offset: {}",
                hex::HexString(offsets.game_scene_node.origin));
        } else if (name == "m_modelState") {
            if (offsets.game_scene_node.model_state != 0) {
                continue;
            }
            offsets.game_scene_node.model_state = *reinterpret_cast<i32 *>(entry + 0x08);
            logging::Debug(
                "player skeleton netvar offset: {}",
                hex::HexString(offsets.game_scene_node.model_state));
        } else if (name == "m_bSpotted") {
            if (offsets.spotted_state.spotted != 0) {
                continue;
            }
            offsets.spotted_state.spotted = *reinterpret_cast<i32 *>(entry + 0x10);
            logging::Debug(
                "player spotted netvar offset: {}", hex::HexString(offsets.spotted_state.spotted));
        } else if (name == "m_bSpottedByMask") {
            if (!network_enable || offsets.spotted_state.mask != 0) {
                continue;
            }
            offsets.spotted_state.mask = *reinterpret_cast<i32 *>(entry + 0x18);
            logging::Debug(
                "player spotted mask netvar offset: {}",
                hex::HexString(offsets.spotted_state.mask));
        } else if (name == "m_hObserverTarget") {
            if (offsets.observer_service.target != 0) {
                continue;
            }
            offsets.observer_service.target = *reinterpret_cast<i32 *>(entry + 0x08);
            logging::Debug(
                "player observer target netvar offset: {}",
                hex::HexString(offsets.observer_service.target));
        } else if (name == "m_iFOV") {
            if (offsets.camera_service.fov != 0) {
                continue;
            }
            offsets.camera_service.fov = *reinterpret_cast<i32 *>(entry + 0x08);
            logging::Debug(
                "player fov netvar offset: {}", hex::HexString(offsets.camera_service.fov));
        } else if (name == "m_bHasDefuser") {
            if (offsets.item_service.has_defuser != 0) {
                continue;
            }
            offsets.item_service.has_defuser = *reinterpret_cast<i32 *>(entry + 0x10);
            logging::Debug(
                "player defuser netvar offset: {}",
                hex::HexString(offsets.item_service.has_defuser));
        } else if (name == "m_bHasHelmet") {
            if (!network_enable || offsets.item_service.has_helmet != 0) {
                continue;
            }
            offsets.item_service.has_helmet = *reinterpret_cast<i32 *>(entry + 0x18);
            logging::Debug(
                "player helmet netvar offset: {}", hex::HexString(offsets.item_service.has_helmet));
        } else if (name == "m_hMyWeapons") {
            if (offsets.weapon_service.weapons != 0) {
                continue;
            }
            offsets.weapon_service.weapons = *reinterpret_cast<i32 *>(entry + 0x08);
            logging::Debug(
                "player weapons netvar offset: {}", hex::HexString(offsets.weapon_service.weapons));
        } else if (name == "m_bC4Activated") {
            if (offsets.planted_c4.is_activated != 0) {
                continue;
            }
            offsets.planted_c4.is_activated = *reinterpret_cast<i32 *>(entry + 0x10);
            logging::Debug(
                "planted c4 activated netvar offset: {}",
                hex::HexString(offsets.planted_c4.is_activated));
        } else if (name == "m_bBombTicking") {
            if (offsets.planted_c4.is_ticking != 0) {
                continue;
            }
            offsets.planted_c4.is_ticking = *reinterpret_cast<i32 *>(entry + 0x10);
            logging::Debug(
                "planted c4 ticking netvar offset: {}",
                hex::HexString(offsets.planted_c4.is_ticking));
        } else if (name == "m_nBombSite") {
            if (!network_enable || offsets.planted_c4.bomb_site != 0) {
                continue;
            }
            offsets.planted_c4.bomb_site = *reinterpret_cast<i32 *>(entry + 0x18);
            logging::Debug(
                "planted c4 bomb site netvar offset: {}",
                hex::HexString(offsets.planted_c4.bomb_site));
        } else if (name == "m_flC4Blow") {
            if (offsets.planted_c4.blow_time != 0) {
                continue;
            }
            offsets.planted_c4.blow_time = *reinterpret_cast<i32 *>(entry + 0x10);
            logging::Debug(
                "planted c4 blow time netvar offset: {}",
                hex::HexString(offsets.planted_c4.blow_time));
        } else if (name == "m_bBeingDefused") {
            if (!network_enable || offsets.planted_c4.being_defused != 0) {
                continue;
            }
            offsets.planted_c4.being_defused = *reinterpret_cast<i32 *>(entry + 0x18);
            logging::Debug(
                "planted c4 defusing netvar offset: {}",
                hex::HexString(offsets.planted_c4.being_defused));
        }

        if (offsets.AllFound()) {
            logging::Info("offsets found");
            target.Reset();
            return offsets;
        }
    }

    logging::Error("did not find all offsets");
    return std::nullopt;
}

f32 Sensitivity() { return process.Read<f32>(offsets.convar.sensitivity + 0x40); }

bool IsFfa() { return process.Read<bool>(offsets.convar.ffa + 0x40); }

bool EntityHasOwner(const u64 entity) {
    // h_pOwnerEntity is a handle, which is an int
    return process.Read<i32>(entity + offsets.controller.owner_entity) != -1;
}

std::optional<std::string> GetEntityType(const u64 entity) {
    // m_pEntity
    const u64 entity_instance = process.Read<u64>(entity + 0x10);
    if (!entity_instance) {
        return std::nullopt;
    }

    // m_designerName
    const u64 name_pointer = process.Read<u64>(entity_instance + 0x20);
    if (!name_pointer) {
        return std::nullopt;
    }

    std::string entity_name = process.ReadString(name_pointer);

    if (entity_name.compare(0, 7, "weapon_") == 0) {
        entity_name = entity_name.erase(0, 7);
        return entity_name;
    }

    return std::nullopt;
}

bool IsButtonPressed(const KeyCode &button) {
    // what the actual fuck is happening here?
    const u32 value = process.Read<u32>(
        offsets.interface.input + ((button >> 5) * 4 + offsets.direct.button_state));
    return (value >> (button & 31) & 1) != 0;
}

glm::vec2 TargetAngle(
    const glm::vec3 &eye_position, const glm::vec3 &position, const glm::vec2 &aim_punch) {
    const glm::vec3 forward = normalize(position - eye_position);
    glm::vec2 angles = AnglesFromVector(forward) - aim_punch;
    Vec2Clamp(angles);
    return angles;
}

// 5.0 fov scale at 0 units, down to 1.0 at 500 units and above
f32 DistanceScale(const f32 distance) {
    if (distance > 500.0f) {
        return 1.0f;
    }
    return 5.0f - distance / 125.0f;
}

std::string MapName() {
    const u64 map_name_ptr = process.Read<u64>(offsets.direct.game_types + 288);
    const std::string map_name = process.ReadString(map_name_ptr);
    return map_name.length() > 4 ? map_name.substr(4) : "";
}

bool FindTarget() {
    const std::optional<Player> local_player = Player::LocalPlayer();
    if (!local_player) {
        return false;
    }

    const u8 local_team = local_player->Team();
    if (local_team != TEAM_CT && local_team != TEAM_T) {
        return false;
    }

    const bool ffa = IsFfa();

    // note to self: forgetting to clear this caused such a retarded memory leak
    players.clear();
    for (u64 i = 1; i <= 64; i++) {
        const std::optional<Player> player = Player::Index(i);
        if (!player) {
            continue;
        }

        if (!player->IsValid()) {
            continue;
        }

        if (player->Equals(*local_player)) {
            target.local_pawn_index = i - 1;
        }

        if (!ffa && player->Team() == local_team) {
            continue;
        }

        players.push_back(*player);
    }

    if (players.empty()) {
        target.Reset();
        return false;
    }

    const WeaponClass weapon_class = local_player->GetWeaponClass();
    if (weapon_class == WeaponClass::Unknown || weapon_class == WeaponClass::Grenade) {
        target.Reset();
        return true;
    }

    const glm::vec2 view_angles = local_player->ViewAngles();
    const i32 shots_fired = local_player->ShotsFired();
    glm::vec2 punch {0.0f};
    if (weapon_class != WeaponClass::Sniper) {
        const glm::vec2 p = local_player->AimPunch();
        if (length(p) < 0.001f && shots_fired > 0) {
            punch = aim_punch;
        } else {
            punch = p;
        }
    }
    aim_punch = punch;

    f32 smallest_fov {360.0f};
    const glm::vec3 eye_position = local_player->EyePosition();
    if (target.player) {
        if (!target.player->IsValid()) {
            target.Reset();
        }
    } else {
        target.Reset();
    }

    if (AnglesToFov(view_angles, target.angle) >
        config.aimbot.fov * DistanceScale(target.distance)) {
        target.Reset();
    }

    // update target player
    if (!IsButtonPressed(config.aimbot.hotkey) || !target.player) {
        for (const Player &player : players) {
            if (!ffa && local_team == player.Team()) {
                continue;
            }

            const glm::vec3 head_position = player.BonePosition(Bones::Head);
            const f32 distance = glm::distance(eye_position, head_position);
            const glm::vec2 angle = TargetAngle(eye_position, head_position, aim_punch);
            const f32 fov = AnglesToFov(view_angles, angle);

            if (fov < smallest_fov) {
                smallest_fov = fov;

                target.player = player;
                target.angle = angle;
                target.distance = distance;
                target.bone_index = Bones::Head;
            }
        }
    }

    if (!target.player) {
        return true;
    }

    // update target angle
    if (config.aimbot.multibone) {
        smallest_fov = 360.0f;
        for (const Bones bone : all_bones) {
            const glm::vec3 bone_position = target.player->BonePosition(bone);
            const f32 distance = glm::distance(eye_position, bone_position);
            const glm::vec2 angle = TargetAngle(eye_position, bone_position, aim_punch);
            const f32 fov = AnglesToFov(view_angles, angle);

            if (fov < smallest_fov) {
                smallest_fov = fov;

                target.angle = angle;
                target.distance = distance;
                target.bone_index = bone;
            }
        }
    }

    return true;
}

void ClearVisualInfo() {
    vinfo_lock.lock();
    players.clear();
    entity_info.clear();
    vinfo_lock.unlock();
}

void VisualInfo() {
    vinfo_lock.lock();
    const std::optional<Player> local_player = Player::LocalPlayer();
    if (local_player->Team() != TEAM_CT && local_player->Team() != TEAM_T) {
        player_info.clear();
        entity_info.clear();
        vinfo_lock.unlock();
        misc_info.in_game = false;
        return;
    }
    misc_info.in_game = true;
    const std::optional<u64> spectated_player = local_player->SpectatorTarget();
    player_info.clear();
    for (const Player &player : players) {
        if ((spectated_player && player.controller == spectated_player) ||
            player.Equals(*local_player)) {
            continue;
        }
        PlayerInfo info = {};
        info.health = player.Health();
        info.armor = player.Armor();
        info.team = player.Team();
        info.position = player.Position();
        info.head = player.BonePosition(Bones::Head);
        info.has_defuser = player.HasDefuser();
        info.has_helmet = player.HasHelmet();
        info.has_bomb = player.HasBomb();
        info.name = player.Name();
        info.weapon = player.WeaponName();
        info.weapons = player.AllWeapons();
        info.bones = player.AllBones();

        player_info.push_back(info);
    }

    // entities
    entity_info.clear();
    for (u64 i = 64; i <= 1024; i++) {
        // entity is a pawn here
        const std::optional<u64> entity = Player::ClientEntity(i);
        if (!entity) {
            continue;
        }

        // is a held weapon
        if (EntityHasOwner(*entity)) {
            continue;
        }

        const std::optional<std::string> name = GetEntityType(*entity);
        if (!name) {
            continue;
        }

        const u64 gs_node = process.Read<u64>(*entity + offsets.pawn.game_scene_node);
        if (!gs_node) {
            continue;
        }
        const auto position = process.Read<glm::vec3>(gs_node + offsets.game_scene_node.origin);

        entity_info.push_back(EntityInfo {.name = *name, .position = position});
    }

    view_matrix = process.Read<glm::mat4>(offsets.direct.view_matrix);
    const u64 sdl_window = process.Read<u64>(offsets.direct.sdl_window);
    if (sdl_window == 0) {
        window_size = glm::ivec4 {0};
    } else {
        window_size = process.Read<glm::ivec4>(sdl_window + 0x18);
    }

    if (local_player) {
        misc_info.held_weapon = local_player->WeaponName();
    } else {
        misc_info.held_weapon = "?";
    }
    misc_info.is_ffa = IsFfa();
    misc_info.map_name = MapName();
    vinfo_lock.unlock();
}

void Run() {
    VisualInfo();
    FindTarget();
    if (!Player::LocalPlayer().has_value()) {
        ClearVisualInfo();
        return;
    }

    FovChanger();
    NoFlash();
    Rcs();

    Aimbot();
    Triggerbot();
}
