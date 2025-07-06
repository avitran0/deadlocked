#include <glm/glm.hpp>
#include <mithril/logging.hpp>
#include <mutex>
#include <vector>

#include "config.hpp"
#include "cs2/info.hpp"

std::mutex config_lock;
std::string current_config = "deadlocked.toml";
std::vector<std::string> available_configs = ListConfigs();
Config config;

std::mutex vinfo_lock;
std::vector<PlayerInfo> player_info {32};
std::vector<EntityInfo> entity_info {128};
PlayerInfo local_player;
glm::mat4 view_matrix;
glm::vec4 window_size;
MiscInfo misc_info;
Flags flags;
