#include "cs2/cs2.hpp"
#include "cs2/features.hpp"
#include "mouse.hpp"

glm::vec2 mouse_movement {0.0f};

void Rcs() {
    if (!config.aimbot.rcs) {
        return;
    }

    const std::optional<Player> local_player = Player::LocalPlayer();
    if (!local_player) {
        return;
    }

    const WeaponClass weapon_class = local_player->GetWeaponClass();
    if (weapon_class != WeaponClass::Smg && weapon_class != WeaponClass::Rifle &&
        weapon_class != WeaponClass::Heavy) {
        return;
    }

    const i32 shots_fired = local_player->ShotsFired();

    if (shots_fired < 1) {
        mouse_movement = glm::vec2 {0.0f};
        return;
    }

    if (glm::length(target.aim_punch) < 0.01f) {
        return;
    }

    const f32 sensitivity = Sensitivity() * local_player->FovMultiplier();

    const glm::vec2 mouse_angle {
        target.aim_punch.y / sensitivity * 25.0f, -target.aim_punch.x / sensitivity * 25.0f};
    const glm::vec2 delta = mouse_angle - mouse_movement;

    mouse_movement += glm::round(delta);

    MouseMove(glm::ivec2 {static_cast<i32>(delta.x), static_cast<i32>(delta.y)});
}
