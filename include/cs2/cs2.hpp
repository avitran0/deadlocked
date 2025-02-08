#pragma once

#include <glm/glm.hpp>
#include <optional>
#include <process.hpp>

#include "config.hpp"
#include "cs2/bones.hpp"
#include "cs2/offsets.hpp"
#include "cs2/player.hpp"

struct PlayerInfo {
    i32 health;
    i32 armor;
    u8 team;
    glm::vec3 position;
    glm::vec3 head;
    bool visible;
    std::string weapon;
    std::vector<std::pair<glm::vec3, glm::vec3>> bones;
};

struct Target {
    std::optional<Player> player;
    Bones bone_index;
    glm::vec2 angle;
    f32 distance;
    u64 local_pawn_index;
    glm::vec2 previous_aim_punch;

    void Reset() {
        player = std::nullopt;
        bone_index = Bones::BonePelvis;
        angle = glm::vec2(0.0);
        distance = 0.0;
        local_pawn_index = 0;
        previous_aim_punch = glm::vec2(0.0);
    }
};

extern Config config;
extern Process process;
extern Offsets offsets;

void CS2();
bool IsValid();
void Setup();
void Run();

std::optional<Offsets> FindOffsets();

f32 Sensitivity();
bool IsFfa();
bool EntityHasOwner(const u64 entity);
std::optional<std::string> GetEntityType(const u64 entity);
