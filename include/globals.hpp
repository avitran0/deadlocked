#pragma once

#include <mutex>

#include "config.hpp"
#include "cs2/info.hpp"
#include "version_check.hpp"

extern std::mutex config_lock;
extern Config config;

extern std::mutex vinfo_lock;
extern std::vector<PlayerInfo> player_info;
extern std::vector<EntityInfo> entity_info;
extern PlayerInfo local_player;
extern glm::mat4 view_matrix;
extern glm::vec4 window_size;
extern MiscInfo misc_info;
extern Flags flags;
extern bool new_version;
extern SystemInfo system_info;
