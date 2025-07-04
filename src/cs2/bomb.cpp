#include "cs2/bomb.hpp"

#include "cs2/cs2.hpp"

std::optional<Bomb> Bomb::Get() {
    const u64 bomb_handle = process.Read<u64>(offsets.direct.planted_c4);
    if (!bomb_handle) {
        return std::nullopt;
    }

    const u64 bomb = process.Read<u64>(bomb_handle);
    if (!bomb) {
        return std::nullopt;
    }

    return Bomb {bomb};
}

bool Bomb::IsPlanted() const {
    return process.Read<bool>(entity + offsets.planted_c4.is_activated);
}

bool Bomb::IsBeingDefused() const {
    return process.Read<bool>(entity + offsets.planted_c4.being_defused);
}

BombSite Bomb::GetBombSite() const {
    const i32 site = process.Read<i32>(entity + offsets.planted_c4.bomb_site);
    if (site < 0 || site > 1) {
        return BombSite::A;
    }
    return static_cast<BombSite>(site);
}

f32 Bomb::TimeToExplosion() const {
    const u64 global_vars = process.Read<u64>(offsets.direct.global_vars);
    const f32 current_time = process.Read<f32>(global_vars + 52);
    return process.Read<f32>(entity + offsets.planted_c4.blow_time) - current_time;
}

glm::vec3 Bomb::Position() const {
    Player bomb = Player {.controller = 0, .pawn = entity};
    return bomb.Position();
}
