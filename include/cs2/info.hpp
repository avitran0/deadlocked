#pragma once

#include <glm/glm.hpp>
#include <mithril/types.hpp>
#include <string>
#include <vector>

struct PlayerInfo {
    glm::vec3 position;
    glm::vec3 head;
    std::string name;
    std::string weapon;
    std::vector<std::string> weapons;
    std::vector<std::pair<glm::vec3, glm::vec3>> bones;

    i32 health;
    i32 armor;

    u8 team;
    bool has_defuser;
    bool has_helmet;
    bool has_bomb;
};

struct EntityInfo {
    std::string name;
    glm::vec3 position;
};

struct MiscInfo {
    std::vector<std::string> spectators;
    std::string held_weapon;
    std::string map_name;
    f32 gui_scale = -1.0f;
    bool in_game;
    bool is_ffa;
    bool triggerbot_active;
};
