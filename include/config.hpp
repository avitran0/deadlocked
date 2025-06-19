#pragma once

#include <imgui.h>

#include <chrono>
#include <mithril/types.hpp>
#include <unordered_map>

#include "colors.hpp"
#include "key_code.hpp"
#include "toml.hpp"

#define VERSION "v6.0.0"

constexpr std::chrono::seconds save_interval {1};

// imvec4 toml helper functions
toml::array imvec4_to_array(const ImVec4 &vec);
ImVec4 array_to_imvec4(const toml::array &arr);

toml::array imvec2_to_array(const ImVec2 &vec);
ImVec2 array_to_imvec2(const toml::array &arr);

enum class Position : u8 {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
};

const std::map<Position, const char *> position_names = {
    {Position::TopLeft, "Top Left"},
    {Position::TopRight, "Top Right"},
    {Position::BottomLeft, "Bottom Left"},
    {Position::BottomRight, "Bottom Right"},
};

enum class DrawStyle : u8 {
    None,
    Color,
    Health,
};

const std::map<DrawStyle, const char *> draw_style_names = {
    {DrawStyle::None, "None"},
    {DrawStyle::Color, "Color"},
    {DrawStyle::Health, "Health"},
};

constexpr i32 DEFAULT_FOV = 90;

struct WeaponConfig {
    i32 start_bullet = 2;
    f32 fov = 2.5f;
    f32 smooth = 5.0f;

    bool enabled = false;
    bool aim_lock = false;
    bool visibility_check = true;
    bool multibone = true;
    bool flash_check = true;

    toml::table to_toml() const;
    static WeaponConfig from_toml(const toml::table &table);

    WeaponConfig() {}
    WeaponConfig(bool enabled) : enabled(enabled) {}
};

struct AimbotConfig {
    std::unordered_map<std::string, WeaponConfig> weapons =
        {  // Knives
            {"bayonet", WeaponConfig()},
            {"knife", WeaponConfig()},
            {"knife_bowie", WeaponConfig()},
            {"knife_butterfly", WeaponConfig()},
            {"knife_canis", WeaponConfig()},
            {"knife_cord", WeaponConfig()},
            {"knife_css", WeaponConfig()},
            {"knife_falchion", WeaponConfig()},
            {"knife_flip", WeaponConfig()},
            {"knife_gut", WeaponConfig()},
            {"knife_gypsy_jackknife", WeaponConfig()},
            {"knife_karambit", WeaponConfig()},
            {"knife_kukri", WeaponConfig()},
            {"knife_m9_bayonet", WeaponConfig()},
            {"knife_outdoor", WeaponConfig()},
            {"knife_push", WeaponConfig()},
            {"knife_skeleton", WeaponConfig()},
            {"knife_stiletto", WeaponConfig()},
            {"knife_survival_bowie", WeaponConfig()},
            {"knife_t", WeaponConfig()},
            {"knife_tactical", WeaponConfig()},
            {"knife_twinblade", WeaponConfig()},
            {"knife_ursus", WeaponConfig()},
            {"knife_widowmaker", WeaponConfig()},

            // Pistols
            {"cz75a", WeaponConfig()},
            {"deagle", WeaponConfig()},
            {"elite", WeaponConfig()},
            {"fiveseven", WeaponConfig()},
            {"glock", WeaponConfig()},
            {"hkp2000", WeaponConfig()},
            {"p2000", WeaponConfig()},
            {"p250", WeaponConfig()},
            {"revolver", WeaponConfig()},
            {"tec9", WeaponConfig()},
            {"usp_silencer", WeaponConfig()},
            {"usp_silencer_off", WeaponConfig()},

            // SMGs
            {"bizon", WeaponConfig()},
            {"mac10", WeaponConfig()},
            {"mp5sd", WeaponConfig()},
            {"mp7", WeaponConfig()},
            {"mp9", WeaponConfig()},
            {"p90", WeaponConfig()},
            {"ump45", WeaponConfig()},

            // Heavy
            {"m249", WeaponConfig()},
            {"negev", WeaponConfig()},

            // Shotguns
            {"mag7", WeaponConfig()},
            {"nova", WeaponConfig()},
            {"sawedoff", WeaponConfig()},
            {"xm1014", WeaponConfig()},

            // Rifles
            {"ak47", WeaponConfig()},
            {"aug", WeaponConfig()},
            {"famas", WeaponConfig()},
            {"galilar", WeaponConfig()},
            {"m4a1_silencer", WeaponConfig()},
            {"m4a1_silencer_off", WeaponConfig()},
            {"m4a1", WeaponConfig()},
            {"sg556", WeaponConfig()},

            // Snipers
            {"awp", WeaponConfig()},
            {"g3sg1", WeaponConfig()},
            {"scar20", WeaponConfig()},
            {"ssg08", WeaponConfig()},

            // Grenades
            {"decoy", WeaponConfig()},
            {"firebomb", WeaponConfig()},
            {"flashbang", WeaponConfig()},
            {"frag_grenade", WeaponConfig()},
            {"hegrenade", WeaponConfig()},
            {"incgrenade", WeaponConfig()},
            {"molotov", WeaponConfig()},
            {"smokegrenade", WeaponConfig()},

            // Utility
            {"taser", WeaponConfig()}};
    WeaponConfig global = WeaponConfig(true);
    KeyCode hotkey = KeyCode::Mouse5;

    bool fov_circle = false;
    bool rcs = false;

    toml::table to_toml() const;
    static AimbotConfig from_toml(const toml::table &table);

    WeaponConfig &GetWeaponConfig(const std::string &name);
    WeaponConfig &CurrentWeaponConfig(const std::string &name);
};

struct TriggerbotConfig {
    ImVec2 indicator_inset {0.0f, 0.0f};

    KeyCode hotkey = KeyCode::Mouse4;
    i32 delay_min = 100;
    i32 delay_max = 200;
    f32 velocity_threshold = 100.0f;

    Position indicator_position = Position::BottomLeft;
    bool enabled = false;
    bool visibility_check = true;
    bool flash_check = true;
    bool scope_check = true;
    bool head_only = false;
    bool toggle_mode = false;
    bool velocity_check = false;

    toml::table to_toml() const;
    static TriggerbotConfig from_toml(const toml::table &table);
};

struct VisualsConfig {
    ImVec4 text_color {1.0f, 1.0f, 1.0f, 1.0f};
    ImVec4 box_color {1.0f, 1.0f, 1.0f, 1.0f};
    ImVec4 skeleton_color {1.0f, 1.0f, 1.0f, 1.0f};
    ImVec4 armor_color {0.0f, 0.0f, 1.0f, 1.0f};
    ImVec4 crosshair_color {1.0f, 1.0f, 1.0f, 1.0f};

    i32 overlay_fps = 120;
    f32 line_width = 2.0;
    f32 font_size = 16.0;

    DrawStyle draw_box = DrawStyle::Color;
    DrawStyle draw_skeleton = DrawStyle::Health;
    bool enabled = true;
    bool draw_health = true;
    bool draw_armor = true;
    bool draw_name = true;
    bool draw_weapon = true;
    bool draw_tags = true;
    bool dropped_weapons = true;
    bool sniper_crosshair = true;
    bool spectator_list = true;
    bool debug_window = false;

    toml::table to_toml() const;
    static VisualsConfig from_toml(const toml::table &table);
};

struct MiscConfig {
    f32 max_flash_alpha = 0.0f;
    i32 desired_fov = DEFAULT_FOV;

    bool no_flash = false;
    bool fov_changer = false;

    toml::table to_toml() const;
    static MiscConfig from_toml(const toml::table &table);
};

struct Config {
    AimbotConfig aimbot;
    TriggerbotConfig triggerbot;
    VisualsConfig visuals;
    MiscConfig misc;

    ImVec4 accent_color = Colors::BLUE;

    toml::table to_toml() const;
    static Config from_toml(const toml::table &table);
};

struct Flags {
    bool should_quit = false;
    // whether or not to read memory from file or via process_vm_readv
    bool file_mem = false;
    bool no_visuals = false;
};

void SaveConfig();
Config LoadConfig();
void ResetConfig();
