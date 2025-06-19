#include <chrono>
#include <random>

#include "cs2/cs2.hpp"
#include "cs2/features.hpp"
#include "math.hpp"
#include "mouse.hpp"

std::optional<std::chrono::steady_clock::time_point> next_shot = std::nullopt;
bool previous_key_state = false;
bool toggled = false;

void Triggerbot() {
    if (!config.triggerbot.enabled) {
        return;
    }

    const bool key_state = IsButtonPressed(config.triggerbot.hotkey);
    if (config.triggerbot.toggle_mode) {
        // toggle mode
        if (key_state && !previous_key_state) {
            toggled = !toggled;
        }
        previous_key_state = key_state;
        misc_info.triggerbot_active = toggled;
    } else {
        // default mode, key has to be held
        misc_info.triggerbot_active = key_state;
    }

    if (!misc_info.triggerbot_active) {
        return;
    }

    if (next_shot) {
        const auto time = *next_shot;
        const auto now = std::chrono::steady_clock::now();
        if (now > time) {
            MouseLeftPress();
            MouseLeftRelease();
            next_shot = std::nullopt;
        }
        return;
    }

    const std::optional<Player> local_player = Player::LocalPlayer();
    if (!local_player) {
        return;
    }

    if (config.triggerbot.flash_check && local_player->IsFlashed()) {
        return;
    }

    if (config.triggerbot.scope_check && local_player->GetWeaponClass() == WeaponClass::Sniper &&
        !local_player->IsScoped()) {
        return;
    }

    const glm::vec3 velocity = local_player->Velocity();
    if (config.triggerbot.velocity_check &&
        glm::length(velocity) > config.triggerbot.velocity_threshold) {
        return;
    }

    const std::optional<Player> crosshair_entity = local_player->EntityInCrosshair();
    if (!crosshair_entity) {
        return;
    }

    if (config.triggerbot.head_only) {
        const glm::vec3 enemy_head = target.player->BonePosition(Bones::Head);
        const glm::vec3 crosshair_pos = local_player->EyePosition();

        const glm::vec2 target_angle = TargetAngle(crosshair_pos, enemy_head, glm::vec2(0.0f));

        const glm::vec2 view_angles = local_player->ViewAngles();

        const f32 fov_distance = AnglesToFov(view_angles, target_angle);

        constexpr f32 head_radius_world = 3.5f;
        const f32 head_radius_fov = head_radius_world / target.distance * 100.0f;

        if (fov_distance > head_radius_fov) {
            return;
        }
    }

    if (!IsFfa() && crosshair_entity->Team() == local_player->Team()) {
        return;
    }

    std::random_device dev;
    std::mt19937 rng {dev()};
    const f32 mean =
        static_cast<f32>(config.triggerbot.delay_min + config.triggerbot.delay_max) / 2.0f;
    std::normal_distribution normal {
        mean, static_cast<f32>(config.triggerbot.delay_max - config.triggerbot.delay_min) / 2.0f};

    const i32 delay = static_cast<i32>(normal(rng));
    next_shot = std::chrono::steady_clock::now() + std::chrono::milliseconds(delay);
}
