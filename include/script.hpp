#pragma once

#include <glm/glm.hpp>

extern "C" {
#include <lauxlib.h>
}

#include <string>
#include <utility>
#include <vector>

#include "key_code.hpp"
#include "types.hpp"

struct Vec2 {
    f32 x, y;

    Vec2 FromGlm(const glm::vec2 &vec);
    glm::vec2 ToGlm();
};

struct Vec3 {
    f32 x, y, z;

    Vec3 FromGlm(const glm::vec3 &vec);
    glm::vec3 ToGlm();
};

struct Script {
    lua_State *state;
    std::vector<std::string> scripts;

    std::vector<int> exec_once;
    std::vector<int> exec_tick;
    std::vector<std::pair<KeyCode, int>> exec_key_held;
    std::vector<std::pair<KeyCode, int>> exec_key_pressed;

    Script();

  private:
    void RunFunction(i32 ref);
};

extern Script script;
