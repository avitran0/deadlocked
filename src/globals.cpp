#include <filesystem>
#include <fstream>
#include <glm/glm.hpp>
#include <vector>
#include <nlohmann/json.hpp>

#include "config.hpp"
#include "cs2/cs2.hpp"

using json = nlohmann::json;

Config LoadConfig();

std::mutex config_lock;
Config config = LoadConfig();

std::mutex vinfo_lock;
std::vector<PlayerInfo> player_info;
std::vector<EntityInfo> entity_info;
glm::mat4 view_matrix;
glm::ivec4 window_size;
bool should_quit = false;

// Convert ImVec4 to JSON
void to_json(json& j, const ImVec4& v) {
    j = json{{"x", v.x}, {"y", v.y}, {"z", v.z}, {"w", v.w}};
}

void from_json(const json& j, ImVec4& v) {
    j.at("x").get_to(v.x);
    j.at("y").get_to(v.y);
    j.at("z").get_to(v.z);
    j.at("w").get_to(v.w);
}

Config DefaultConfig() {
    return Config{.aimbot =
                      {
                          .hotkey = KeyCode::Mouse5,
                          .start_bullet = 2,
                          .fov = 2.5f,
                          .smooth = 5.0f,

                          .enabled = true,
                          .aim_lock = false,
                          .visibility_check = true,
                          .multibone = true,
                          .flash_check = true,
                          .rcs = false,
                      },
                  .visuals =
                      {
                          .box_color = ImVec4(1.0f, 1.0f, 1.0f, 1.0f),
                          .skeleton_color = ImVec4(1.0f, 1.0f, 1.0f, 1.0f),
                          .armor_color = ImVec4(0.0f, 0.0f, 1.0f, 1.0f),

                          .overlay_fps = 120,
                          .line_width = 2.0f,

                          .draw_box = DrawStyle::DrawColor,
                          .draw_skeleton = DrawStyle::DrawHealth,
                          .enabled = true,
                          .draw_health = true,
                          .draw_armor = true,
                          .draw_name = true,
                          .draw_weapon = true,
                          .draw_tags = true,
                          .dropped_weapons = true,
                          .debug_window = false,
                      },
                  .misc = {
                      .max_flash_alpha = 0.0f,
                      .desired_fov = 90,

                      .no_flash = false,
                      .fov_changer = false,
                  }};
}

std::string ConfigPath() {
    const auto exe = std::filesystem::canonical("/proc/self/exe");
    const auto exe_path = exe.parent_path();
    return (exe_path / std::filesystem::path("deadlocked.json")).string();
}

void SaveConfig() {
    json j;

    // Convert Config to JSON
    j["aimbot"] = {
        {"hotkey", static_cast<int>(config.aimbot.hotkey)},
        {"start_bullet", config.aimbot.start_bullet},
        {"fov", config.aimbot.fov},
        {"smooth", config.aimbot.smooth},
        {"enabled", config.aimbot.enabled},
        {"aim_lock", config.aimbot.aim_lock},
        {"visibility_check", config.aimbot.visibility_check},
        {"multibone", config.aimbot.multibone},
        {"flash_check", config.aimbot.flash_check},
        {"rcs", config.aimbot.rcs}
    };

    j["visuals"] = {
        {"box_color", config.visuals.box_color},
        {"skeleton_color", config.visuals.skeleton_color},
        {"armor_color", config.visuals.armor_color},
        {"overlay_fps", config.visuals.overlay_fps},
        {"line_width", config.visuals.line_width},
        {"draw_box", static_cast<int>(config.visuals.draw_box)},
        {"draw_skeleton", static_cast<int>(config.visuals.draw_skeleton)},
        {"enabled", config.visuals.enabled},
        {"draw_health", config.visuals.draw_health},
        {"draw_armor", config.visuals.draw_armor},
        {"draw_name", config.visuals.draw_name},
        {"draw_weapon", config.visuals.draw_weapon},
        {"draw_tags", config.visuals.draw_tags},
        {"dropped_weapons", config.visuals.dropped_weapons},
        {"debug_window", config.visuals.debug_window}
    };

    j["misc"] = {
        {"max_flash_alpha", config.misc.max_flash_alpha},
        {"desired_fov", config.misc.desired_fov},
        {"no_flash", config.misc.no_flash},
        {"fov_changer", config.misc.fov_changer}
    };

    // Write to file
    std::ofstream file(ConfigPath());
    if (file.is_open()) {
        file << j.dump(4); // Pretty print with 4 spaces indent
        file.close();
    }
}

Config LoadConfig() {
    Config conf = DefaultConfig();
    std::ifstream file(ConfigPath());
    
    if (!file.is_open()) {
        return conf;
    }

    try {
        json j;
        file >> j;

        // Load aimbot settings
        conf.aimbot.hotkey = static_cast<KeyCode>(j["aimbot"]["hotkey"].get<int>());
        conf.aimbot.start_bullet = j["aimbot"]["start_bullet"];
        conf.aimbot.fov = j["aimbot"]["fov"];
        conf.aimbot.smooth = j["aimbot"]["smooth"];
        conf.aimbot.enabled = j["aimbot"]["enabled"];
        conf.aimbot.aim_lock = j["aimbot"]["aim_lock"];
        conf.aimbot.visibility_check = j["aimbot"]["visibility_check"];
        conf.aimbot.multibone = j["aimbot"]["multibone"];
        conf.aimbot.flash_check = j["aimbot"]["flash_check"];
        conf.aimbot.rcs = j["aimbot"]["rcs"];

        // Load visuals settings
        j["visuals"]["box_color"].get_to(conf.visuals.box_color);
        j["visuals"]["skeleton_color"].get_to(conf.visuals.skeleton_color);
        j["visuals"]["armor_color"].get_to(conf.visuals.armor_color);
        conf.visuals.overlay_fps = j["visuals"]["overlay_fps"];
        conf.visuals.line_width = j["visuals"]["line_width"];
        conf.visuals.draw_box = static_cast<DrawStyle>(j["visuals"]["draw_box"].get<int>());
        conf.visuals.draw_skeleton = static_cast<DrawStyle>(j["visuals"]["draw_skeleton"].get<int>());
        conf.visuals.enabled = j["visuals"]["enabled"];
        conf.visuals.draw_health = j["visuals"]["draw_health"];
        conf.visuals.draw_armor = j["visuals"]["draw_armor"];
        conf.visuals.draw_name = j["visuals"]["draw_name"];
        conf.visuals.draw_weapon = j["visuals"]["draw_weapon"];
        conf.visuals.draw_tags = j["visuals"]["draw_tags"];
        conf.visuals.dropped_weapons = j["visuals"]["dropped_weapons"];
        conf.visuals.debug_window = j["visuals"]["debug_window"];

        // Load misc settings
        conf.misc.max_flash_alpha = j["misc"]["max_flash_alpha"];
        conf.misc.desired_fov = j["misc"]["desired_fov"];
        conf.misc.no_flash = j["misc"]["no_flash"];
        conf.misc.fov_changer = j["misc"]["fov_changer"];
    } catch (const json::exception& e) {
        // If there's any error parsing the JSON, return default config
        return DefaultConfig();
    }

    file.close();
    return conf;
}

void ResetConfig() {
    config = DefaultConfig();
    SaveConfig();
}