#pragma once

#include <glm/glm.hpp>
#include <optional>
#include <process.hpp>

#include "cs2/bones.hpp"
#include "cs2/offsets.hpp"
#include "cs2/player.hpp"

struct Target {
    std::optional<Player> player = std::nullopt;
    Bones bone_index = Bones::Head;
    glm::vec2 angle {0.0f};
    f32 distance {0.0f};
    u64 local_pawn_index {0};

    void Reset() {
        player = std::nullopt;
        bone_index = Bones::Pelvis;
        angle = glm::vec2 {0.0f};
        distance = 0.0f;
        local_pawn_index = 0;
    }
};

extern Process process;
extern Offsets offsets;
extern Target target;
extern glm::vec2 aim_punch;

void CS2();
bool IsValid();
void Setup();
void Run();

std::optional<Offsets> FindOffsets();

f32 Sensitivity();
bool IsFfa();
bool EntityHasOwner(u64 entity);
std::optional<std::string> GetEntityType(u64 entity);
bool IsButtonPressed(const KeyCode &button);
glm::vec2 TargetAngle(
    const glm::vec3 &eye_position, const glm::vec3 &position, const glm::vec2 &aim_punch);
f32 DistanceScale(f32 distance);
