#pragma once

#include <mithril/types.hpp>
#include <optional>
#include <vector>

class Smoke {
  public:
    u64 controller;

    static std::optional<Smoke> Index(u64 index);

    void Disable() const;
    void SetColor() const;
};

void Smokes(const std::vector<Smoke> &smokes);
