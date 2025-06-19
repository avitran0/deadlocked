#include "cs2/player.hpp"

#include <optional>

#include "cs2/cs2.hpp"

std::optional<Player> Player::LocalPlayer() {
    const u64 controller = process.Read<u64>(offsets.direct.local_player);
    if (!controller) {
        return std::nullopt;
    }

    const std::optional<u64> pawn = Pawn(controller);
    if (!pawn) {
        return std::nullopt;
    }
    return Player {.controller = controller, .pawn = *pawn};
}

std::optional<u64> Player::ClientEntity(const u64 index) {
    // wtf is this doing, and how?
    const u64 v1 = process.Read<u64>(offsets.interface.entity + 0x08 * (index >> 9) + 0x10);
    if (!v1) {
        return std::nullopt;
    }
    // what?
    const u64 controller = process.Read<u64>(v1 + 120 * (index & 0x1FF));
    if (!controller) {
        return std::nullopt;
    }

    return controller;
}

std::optional<Player> Player::Index(const u64 index) {
    const std::optional<u64> controller = ClientEntity(index);
    if (!controller) {
        return std::nullopt;
    }

    const std::optional<u64> pawn = Pawn(*controller);
    if (!pawn) {
        return std::nullopt;
    }
    return Player {.controller = *controller, .pawn = *pawn};
}

std::optional<u64> Player::Pawn(const u64 controller) {
    const i32 v1 = process.Read<i32>(controller + offsets.controller.pawn);
    if (v1 == -1) {
        return std::nullopt;
    }

    // what the fuck is this doing?
    const u64 v2 = process.Read<u64>(offsets.interface.player + 8 * ((v1 & 0x7FFF) >> 9));
    if (!v2) {
        return std::nullopt;
    }

    // bit-fuckery, why is this needed exactly?
    const u64 entity = process.Read<u64>(v2 + 120 * (v1 & 0x1FF));
    if (!entity) {
        return std::nullopt;
    }

    return entity;
}

i32 Player::Health() const {
    const i32 health = process.Read<i32>(pawn + offsets.pawn.health);
    if (health < 0 || health > 100) {
        return 0;
    }
    return health;
}

i32 Player::Armor() const {
    const i32 armor = process.Read<i32>(pawn + offsets.pawn.armor);
    if (armor < 0 || armor > 100) {
        return 0;
    }
    return armor;
}

std::string Player::Name() const {
    const u64 name_address = process.Read<u64>(controller + offsets.controller.name);
    if (!name_address) {
        return std::string {"?"};
    }

    return process.ReadString(name_address);
}

u64 Player::SteamID() const { return process.Read<u64>(controller + offsets.controller.steam_id); }

u8 Player::Team() const { return process.Read<u8>(pawn + offsets.pawn.team); }

u8 Player::LifeState() const { return process.Read<u8>(pawn + offsets.pawn.life_state); }

std::string Player::WeaponName() const {
    // CEntityInstance
    const u64 weapon_entity_instance = process.Read<u64>(pawn + offsets.pawn.weapon);
    if (!weapon_entity_instance) {
        return std::string {"?"};
    }
    // CEntityIdentity, 0x10 = m_pEntity
    const u64 weapon_entity_identity = process.Read<u64>(weapon_entity_instance + 0x10);
    if (!weapon_entity_identity) {
        return std::string {"?"};
    }
    // 0x20 = m_designerName (pointer -> string)
    const u64 weapon_name_pointer = process.Read<u64>(weapon_entity_identity + 0x20);
    if (!weapon_name_pointer) {
        return std::string {"?"};
    }

    std::string name = process.ReadString(weapon_name_pointer);
    if (name.find("weapon_") != std::string::npos) {
        name = name.substr(7);
    } else {
        name = "?";
    }
    return name;
}

std::vector<std::string> Player::AllWeapons() const {
    std::vector<std::string> weapons;
    const u64 weapon_service = process.Read<u64>(pawn + offsets.pawn.weapon_services);
    if (!weapon_service) {
        return weapons;
    }
    const u64 length = process.Read<u64>(weapon_service + offsets.weapon_service.weapons);
    const u64 weapon_list =
        process.Read<u64>(weapon_service + offsets.weapon_service.weapons + 0x08);
    for (u64 i = 0; i < length; i++) {
        const u64 weapon_index = process.Read<u32>(weapon_list + 0x04 * i) & 0xFFF;
        // CEntityInstance
        const std::optional<u64> weapon_entity_instance = ClientEntity(weapon_index);
        if (!weapon_entity_instance) {
            continue;
        }
        // CEntityIdentity, 0x10 = m_pEntity
        const u64 weapon_entity_identity = process.Read<u64>(*weapon_entity_instance + 0x10);
        if (!weapon_entity_identity) {
            continue;
        }
        // 0x20 = m_designerName (pointer -> string)
        const u64 weapon_name_pointer = process.Read<u64>(weapon_entity_identity + 0x20);
        if (!weapon_name_pointer) {
            continue;
        }

        const std::string name = process.ReadString(weapon_name_pointer);
        if (name.find("weapon_") != std::string::npos) {
            weapons.push_back(name.substr(7));
        }
    }

    return weapons;
}

WeaponClass Player::GetWeaponClass() const { return WeaponClassFromString(WeaponName()); }

u64 Player::GameSceneNode() const { return process.Read<u64>(pawn + offsets.pawn.game_scene_node); }

bool Player::IsDormant() const {
    return process.Read<u8>(GameSceneNode() + offsets.game_scene_node.dormant) != 0;
}

glm::vec3 Player::Position() const {
    return process.Read<glm::vec3>(GameSceneNode() + offsets.game_scene_node.origin);
}

glm::vec3 Player::EyePosition() const {
    const glm::vec3 position = Position();
    const auto eye_offset = process.Read<glm::vec3>(pawn + offsets.pawn.eye_offset);

    return position + eye_offset;
}

glm::vec3 Player::BonePosition(Bones bone_index) const {
    const u64 bone_data =
        process.Read<u64>(GameSceneNode() + offsets.game_scene_node.model_state + 0x80);

    if (bone_data == 0) {
        return glm::vec3(0.0f);
    }

    return process.Read<glm::vec3>(bone_data + static_cast<u64>(bone_index) * 32);
}

f32 Player::Rotation() const { return process.Read<f32>(pawn + offsets.pawn.eye_angles + 0x04); }

i32 Player::ShotsFired() const { return process.Read<i32>(pawn + offsets.pawn.shots_fired); }

f32 Player::FovMultiplier() const { return process.Read<f32>(pawn + offsets.pawn.fov_multiplier); }

u64 Player::SpottedMask() const {
    return process.Read<u64>(pawn + offsets.pawn.spotted_state + offsets.spotted_state.mask);
}

std::vector<std::pair<glm::vec3, glm::vec3>> Player::AllBones() const {
    static std::unordered_map<Bones, glm::vec3> bones {20};
    for (const Bones bone : all_bones) {
        const glm::vec3 position = BonePosition(bone);
        bones[bone] = position;
    }

    static std::vector<std::pair<glm::vec3, glm::vec3>> connections {bone_connections.size()};
    i32 i = 0;
    for (const auto &[first, second] : bone_connections) {
        connections[i] = {bones.at(first), bones.at(second)};
        i++;
    }

    return connections;
}

bool Player::IsValid() const {
    if (IsDormant()) {
        return false;
    }

    if (Health() <= 0) {
        return false;
    }

    if (LifeState() != 0) {
        return false;
    }

    return true;
}

bool Player::IsFlashed() const {
    return process.Read<f32>(pawn + offsets.pawn.flash_duration) > 0.2f;
}

void Player::NoFlash(const f32 max_alpha) const {
    const f32 clamped_alpha = glm::clamp(max_alpha, 0.0f, 255.0f);
    if (process.Read<f32>(pawn + offsets.pawn.flash_alpha) != clamped_alpha) {
        process.Write<f32>(pawn + offsets.pawn.flash_alpha, clamped_alpha);
    }
}

void Player::SetFov(const i32 fov) const {
    const u64 camera_service = process.Read<u64>(pawn + offsets.pawn.camera_services);
    if (!camera_service) {
        return;
    }
    const i32 clamped_fov = glm::clamp(fov, 1, 179);
    if (process.Read<u32>(camera_service + offsets.camera_service.fov) !=
        static_cast<u32>(clamped_fov)) {
        process.Write<i32>(controller + offsets.controller.desired_fov, clamped_fov);
    }
}

glm::vec2 Player::ViewAngles() const {
    return process.Read<glm::vec2>(pawn + offsets.pawn.view_angles);
}

glm::vec2 Player::AimPunch() const {
    const u64 length = process.Read<u64>(pawn + offsets.pawn.aim_punch_cache);
    if (length < 1) {
        return glm::vec2(0.0f);
    }

    const u64 data_address = process.Read<u64>(pawn + offsets.pawn.aim_punch_cache + 0x08);

    return process.Read<glm::vec2>(data_address + (length - 1) * 12) * glm::vec2(2.0f);
}

bool Player::HasDefuser() const {
    const u64 item_service = process.Read<u64>(pawn + offsets.pawn.item_services);
    if (!item_service) {
        return false;
    }
    return process.Read<u8>(item_service + offsets.item_service.has_defuser) != 0;
}

bool Player::HasHelmet() const {
    const u64 item_service = process.Read<u64>(pawn + offsets.pawn.item_services);
    if (!item_service) {
        return false;
    }
    return process.Read<u8>(item_service + offsets.item_service.has_helmet) != 0;
}

bool Player::HasBomb() const {
    const std::vector<std::string> weapons = AllWeapons();
    for (const auto &weapon : weapons) {
        if (weapon == "c4") {
            return true;
        }
    }
    return false;
}

/// returns spectated player controller
std::optional<u64> Player::SpectatorTarget() const {
    const u64 observer_services = process.Read<u64>(pawn + offsets.pawn.observer_services);
    if (!observer_services) {
        return std::nullopt;
    }

    const u64 controller = process.Read<u32>(observer_services + offsets.observer_service.target);
    if (!controller) {
        return std::nullopt;
    }
    return controller;
}

// returns player with pawn only!
std::optional<Player> Player::EntityInCrosshair() const {
    const i32 index = process.Read<i32>(pawn + offsets.pawn.crosshair_entity);
    if (index == -1) {
        return std::nullopt;
    }

    const std::optional<u64> entity = ClientEntity(index);
    if (!entity) {
        return std::nullopt;
    }
    const Player player {.controller = 0, .pawn = *entity};
    if (!player.IsValid()) {
        return std::nullopt;
    }
    return player;
}

bool Player::IsScoped() const { return process.Read<u8>(pawn + offsets.pawn.scoped) != 0; }

glm::vec3 Player::Velocity() const {
    return process.Read<glm::vec3>(pawn+offsets.pawn.velocity);
}

bool Player::IsLocalPlayer() const {
    const u64 local_controller = process.Read<u64>(offsets.direct.local_player);
    if (!local_controller) {
        return false;
    }
    return controller == local_controller;
}
