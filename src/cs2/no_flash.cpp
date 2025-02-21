#include "cs2/cs2.hpp"
#include "cs2/features.hpp"

void NoFlash() {
    const auto local_player_opt = Player::LocalPlayer();
    if (!local_player_opt.has_value()) {
        return;
    }
    Player local_player = local_player_opt.value();

    if (config.misc.no_flash) {
        local_player.NoFlash(config.misc.max_flash_alpha);
    } else {
        local_player.NoFlash(255.0f);
    }
}
