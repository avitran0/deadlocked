#pragma once

#include <glm/glm.hpp>
#include <mithril/types.hpp>
#include <optional>

#include "cs2/bones.hpp"
#include "weapon_class.hpp"

class Player {
  public:
    u64 controller;
    u64 pawn;

    static std::optional<Player> LocalPlayer();
    static std::optional<Player> Index(u64 index);
    static std::optional<u64> Pawn(u64 controller);
    static std::optional<u64> ClientEntity(u64 index);

    i32 Health() const;
    i32 Armor() const;
    std::string Name() const;
    u64 SteamID() const;
    u8 Team() const;
    u8 LifeState() const;
    std::string WeaponName() const;
    std::vector<std::string> AllWeapons() const;
    WeaponClass GetWeaponClass() const;
    u64 GameSceneNode() const;
    bool IsDormant() const;
    glm::vec3 Position() const;
    glm::vec3 EyePosition() const;
    glm::vec3 BonePosition(Bones bone_index) const;
    f32 Rotation() const;
    i32 ShotsFired() const;
    f32 FovMultiplier() const;
    u64 SpottedMask() const;
    std::vector<std::pair<glm::vec3, glm::vec3>> AllBones() const;
    bool IsValid() const;
    bool IsFlashed() const;
    void NoFlash(f32 max_alpha) const;
    void SetFov(i32 fov) const;
    glm::vec2 ViewAngles() const;
    glm::vec2 AimPunch() const;
    bool HasDefuser() const;
    bool HasHelmet() const;
    bool HasBomb() const;
    std::optional<u64> SpectatorTarget() const;
    // returns player with pawn only, no controller set
    std::optional<Player> EntityInCrosshair() const;
    bool IsScoped() const;

    bool Equals(const Player &other) const {
        return pawn != 0 && controller != 0 && pawn == other.pawn && controller == other.controller;
    }

    bool IsLocalPlayer() const {
        auto local = Player::LocalPlayer();
        if (!local.has_value()) return false;
        return this->pawn == local->pawn || this->controller == local->controller;
    }
};
