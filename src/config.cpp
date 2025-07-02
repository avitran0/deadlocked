#include "config.hpp"

#include "mithril/logging.hpp"

const char *DrawStyleName(DrawStyle style) {
    constexpr const char *names[] = {"None", "Color", "Health"};
    return names[static_cast<i32>(style)];
}

toml::array imvec4_to_array(const ImVec4 &vec) {
    toml::array arr;
    arr.push_back(vec.x);
    arr.push_back(vec.y);
    arr.push_back(vec.z);
    arr.push_back(vec.w);
    return arr;
}

ImVec4 array_to_imvec4(const toml::array &arr) {
    ImVec4 vec;
    if (arr.size() >= 4) {
        vec.x = arr[0].value_or(0.0f);
        vec.y = arr[1].value_or(0.0f);
        vec.z = arr[2].value_or(0.0f);
        vec.w = arr[3].value_or(0.0f);
    }
    return vec;
}

toml::array imvec2_to_array(const ImVec2 &vec) {
    toml::array arr;
    arr.push_back(vec.x);
    arr.push_back(vec.y);
    return arr;
}

ImVec2 array_to_imvec2(const toml::array &arr) {
    ImVec2 vec;
    if (arr.size() >= 4) {
        vec.x = arr[0].value_or(0.0f);
        vec.y = arr[1].value_or(0.0f);
    }
    return vec;
}

toml::table WeaponConfig::to_toml() const {
    return toml::table {
        {"start_bullet", start_bullet},
        {"fov", fov},
        {"smooth", smooth},
        {"rcs_smooth", rcs_smooth},
        {"enabled", enabled},
        {"aim_lock", aim_lock},
        {"visibility_check", visibility_check},
        {"multibone", multibone},
        {"flash_check", flash_check},
        {"rcs", rcs},
    };
}

WeaponConfig WeaponConfig::from_toml(const toml::table &table) {
    WeaponConfig cfg;
    cfg.start_bullet = table["start_bullet"].value_or(cfg.start_bullet);
    cfg.fov = table["fov"].value_or(cfg.fov);
    cfg.smooth = table["smooth"].value_or(cfg.smooth);
    cfg.rcs_smooth = table["smooth_rcs"].value_or(cfg.rcs_smooth);
    cfg.enabled = table["enabled"].value_or(cfg.enabled);
    cfg.aim_lock = table["aim_lock"].value_or(cfg.aim_lock);
    cfg.visibility_check = table["visibility_check"].value_or(cfg.visibility_check);
    cfg.multibone = table["multibone"].value_or(cfg.multibone);
    cfg.flash_check = table["flash_check"].value_or(cfg.flash_check);
    cfg.rcs = table["rcs"].value_or(cfg.rcs);
    return cfg;
}

toml::table AimbotConfig::to_toml() const {
    toml::table weapons_table;
    for (const auto &[weapon, conf] : weapons) {
        weapons_table.emplace(weapon, conf.to_toml());
    }
    return toml::table {
        {"weapons", weapons_table},
        {"global", global.to_toml()},
        {"hotkey", static_cast<int>(hotkey)},
        {"fov_circle", fov_circle}};
}

AimbotConfig AimbotConfig::from_toml(const toml::table &table) {
    AimbotConfig cfg;
    if (auto table_weapons = table["weapons"].as_table()) {
        for (const auto &[key, node] : *table_weapons) {
            const std::string weapon_name = std::string(key);
            cfg.weapons[weapon_name] = WeaponConfig::from_toml(*node.as_table());
        }
    }
    if (auto table_global = table["global"].as_table()) {
        cfg.global = WeaponConfig::from_toml(*table_global);
    }
    cfg.hotkey = static_cast<KeyCode>(table["hotkey"].value_or(static_cast<int>(cfg.hotkey)));
    cfg.fov_circle = table["fov_circle"].value_or(cfg.fov_circle);
    return cfg;
}

WeaponConfig &AimbotConfig::GetWeaponConfig(const std::string &name) {
    const auto it = weapons.find(name);
    if (it != weapons.end()) {
        return it->second;
    }
    logging::Error("could not find weapon config for: {}", name);
    return global;
}

WeaponConfig &AimbotConfig::CurrentWeaponConfig(const std::string &name) {
    const auto it = weapons.find(name);
    if (it != weapons.end() && it->second.enabled) {
        return it->second;
    }
    return global;
}

toml::table TriggerbotConfig::to_toml() const {
    return toml::table {
        {"indicator_inset", imvec2_to_array(indicator_inset)},
        {"hotkey", static_cast<int>(hotkey)},
        {"delay_min", delay_min},
        {"delay_max", delay_max},
        {"velocity_threshold", velocity_threshold},
        {"indicator_position", static_cast<int>(indicator_position)},
        {"enabled", enabled},
        {"visibility_check", visibility_check},
        {"flash_check", flash_check},
        {"scope_check", scope_check},
        {"head_only", head_only},
        {"toggle_mode", toggle_mode},
        {"velocity_check", velocity_check}};
}

TriggerbotConfig TriggerbotConfig::from_toml(const toml::table &table) {
    TriggerbotConfig cfg;
    if (const auto arr = table["indicator_inset"].as_array()) {
        cfg.indicator_inset = array_to_imvec2(*arr);
    }

    cfg.hotkey = static_cast<KeyCode>(table["hotkey"].value_or(static_cast<int>(cfg.hotkey)));
    cfg.delay_min = table["delay_min"].value_or(cfg.delay_min);
    cfg.delay_max = table["delay_max"].value_or(cfg.delay_max);
    cfg.velocity_threshold = table["velocity_threshold"].value_or(cfg.velocity_threshold);
    cfg.indicator_position = static_cast<Position>(
        table["indicator_position"].value_or(static_cast<int>(cfg.indicator_position)));
    cfg.enabled = table["enabled"].value_or(cfg.enabled);
    cfg.visibility_check = table["visibility_check"].value_or(cfg.visibility_check);
    cfg.flash_check = table["flash_check"].value_or(cfg.flash_check);
    cfg.scope_check = table["scope_check"].value_or(cfg.scope_check);
    cfg.head_only = table["head_only"].value_or(cfg.head_only);
    cfg.toggle_mode = table["toggle_mode"].value_or(cfg.toggle_mode);
    cfg.velocity_check = table["velocity_check"].value_or(cfg.velocity_check);
    return cfg;
}

toml::table VisualsConfig::to_toml() const {
    return toml::table {
        {"text_color", imvec4_to_array(text_color)},
        {"box_color", imvec4_to_array(box_color)},
        {"skeleton_color", imvec4_to_array(skeleton_color)},
        {"armor_color", imvec4_to_array(armor_color)},
        {"crosshair_color", imvec4_to_array(crosshair_color)},
        {"overlay_fps", overlay_fps},
        {"line_width", line_width},
        {"font_size", font_size},
        {"draw_box", static_cast<int>(draw_box)},
        {"draw_skeleton", static_cast<int>(draw_skeleton)},
        {"enabled", enabled},
        {"draw_health", draw_health},
        {"draw_armor", draw_armor},
        {"draw_name", draw_name},
        {"draw_weapon", draw_weapon},
        {"draw_tags", draw_tags},
        {"dropped_weapons", dropped_weapons},
        {"sniper_crosshair", sniper_crosshair},
        {"spectator_list", spectator_list},
        {"debug_window", debug_window}};
}

VisualsConfig VisualsConfig::from_toml(const toml::table &table) {
    VisualsConfig cfg;
    if (const auto arr = table["text_color"].as_array()) {
        cfg.text_color = array_to_imvec4(*arr);
    }
    if (const auto arr = table["box_color"].as_array()) {
        cfg.box_color = array_to_imvec4(*arr);
    }
    if (const auto arr = table["skeleton_color"].as_array()) {
        cfg.skeleton_color = array_to_imvec4(*arr);
    }
    if (const auto arr = table["armor_color"].as_array()) {
        cfg.armor_color = array_to_imvec4(*arr);
    }
    if (const auto arr = table["crosshair_color"].as_array()) {
        cfg.crosshair_color = array_to_imvec4(*arr);
    }

    cfg.overlay_fps = table["overlay_fps"].value_or(cfg.overlay_fps);
    cfg.line_width = table["line_width"].value_or(cfg.line_width);
    cfg.font_size = table["font_size"].value_or(cfg.font_size);
    cfg.draw_box =
        static_cast<DrawStyle>(table["draw_box"].value_or(static_cast<int>(cfg.draw_box)));
    cfg.draw_skeleton = static_cast<DrawStyle>(
        table["draw_skeleton"].value_or(static_cast<int>(cfg.draw_skeleton)));
    cfg.enabled = table["enabled"].value_or(cfg.enabled);
    cfg.draw_health = table["draw_health"].value_or(cfg.draw_health);
    cfg.draw_armor = table["draw_armor"].value_or(cfg.draw_armor);
    cfg.draw_name = table["draw_name"].value_or(cfg.draw_name);
    cfg.draw_weapon = table["draw_weapon"].value_or(cfg.draw_weapon);
    cfg.draw_tags = table["draw_tags"].value_or(cfg.draw_tags);
    cfg.dropped_weapons = table["dropped_weapons"].value_or(cfg.dropped_weapons);
    cfg.sniper_crosshair = table["sniper_crosshair"].value_or(cfg.sniper_crosshair);
    cfg.spectator_list = table["spectator_list"].value_or(cfg.spectator_list);
    cfg.debug_window = table["debug_window"].value_or(cfg.debug_window);
    return cfg;
}

toml::table MiscConfig::to_toml() const {
    return toml::table {
        {"smoke_color", imvec4_to_array(smoke_color)},
        {"max_flash_alpha", max_flash_alpha},
        {"desired_fov", desired_fov},
        {"no_flash", no_flash},
        {"fov_changer", fov_changer},
        {"no_smoke", no_smoke},
        {"change_smoke_color", change_smoke_color}};
}

MiscConfig MiscConfig::from_toml(const toml::table &table) {
    MiscConfig cfg;
    if (const auto arr = table["smoke_color"].as_array()) {
        cfg.smoke_color = array_to_imvec4(*arr);
    }
    cfg.max_flash_alpha = table["max_flash_alpha"].value_or(cfg.max_flash_alpha);
    cfg.desired_fov = table["desired_fov"].value_or(cfg.desired_fov);
    cfg.no_flash = table["no_flash"].value_or(cfg.no_flash);
    cfg.fov_changer = table["fov_changer"].value_or(cfg.fov_changer);
    cfg.no_smoke = table["no_smoke"].value_or(cfg.no_smoke);
    cfg.change_smoke_color = table["change_smoke_color"].value_or(cfg.change_smoke_color);
    return cfg;
}

toml::table Config::to_toml() const {
    return toml::table {
        {"aimbot", aimbot.to_toml()},
        {"triggerbot", triggerbot.to_toml()},
        {"visuals", visuals.to_toml()},
        {"misc", misc.to_toml()},
        {"accent_color", imvec4_to_array(accent_color)}};
}

Config Config::from_toml(const toml::table &table) {
    Config cfg;
    if (auto table_aimbot = table["aimbot"].as_table()) {
        cfg.aimbot = AimbotConfig::from_toml(*table_aimbot);
    }
    if (auto table_triggerbot = table["triggerbot"].as_table()) {
        cfg.triggerbot = TriggerbotConfig::from_toml(*table_triggerbot);
    }
    if (auto table_visuals = table["visuals"].as_table()) {
        cfg.visuals = VisualsConfig::from_toml(*table_visuals);
    }
    if (auto table_misc = table["misc"].as_table()) {
        cfg.misc = MiscConfig::from_toml(*table_misc);
    }

    if (auto arr = table["accent_color"].as_array()) {
        cfg.accent_color = array_to_imvec4(*arr);
    }
    return cfg;
}
