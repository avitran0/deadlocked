#pragma once

#include <glm/glm.hpp>
#include <mithril/types.hpp>
#include <optional>

enum class BombSite {
    A,
    B,
};

class Bomb {
  public:
    u64 entity;

    static std::optional<Bomb> Get();
    bool IsPlanted() const;
    bool IsBeingDefused() const;
    BombSite GetBombSite() const;
    f32 TimeToExplosion() const;
    glm::vec3 Position() const;
};
