#pragma once

#include <imgui.h>

#include "key_code.hpp"
#include "types.hpp"

#define FONT "resources/JetBrainsMono.ttf"

enum DrawStyle : u8 {
    DrawNone,
    DrawColor,
    DrawHealth,
};

struct AimbotConfig {
    KeyCode hotkey;
    i32 start_bullet;
    f32 fov;
    f32 smooth;

    bool enabled;
    bool aim_lock;
    bool visibility_check;
    bool multibone;
    bool flash_check;
    bool rcs;
};

struct VisualsConfig {
    ImVec4 box_color;
    ImVec4 skeleton_color;
    ImVec4 armor_color;

    i32 overlay_fps;
    f32 line_width;

    DrawStyle draw_box;
    DrawStyle draw_skeleton;
    bool enabled;
    bool draw_health;
    bool draw_armor;
    bool draw_name;
    bool draw_weapon;
    bool draw_tags;
    bool dropped_weapons;
    bool debug_window;
};

struct MiscConfig {
    f32 max_flash_alpha;
    i32 desired_fov;

    bool no_flash;
    bool fov_changer;
};

struct Config {
    AimbotConfig aimbot;
    VisualsConfig visuals;
    MiscConfig misc;
};

void SaveConfig();
void ResetConfig();
