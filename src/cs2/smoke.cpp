#include "cs2/smoke.hpp"

#include "cs2/cs2.hpp"

std::optional<Smoke> Smoke::Index(const u64 index) {
    // wtf is this doing, and how?
    const u64 v1 = process.Read<u64>(offsets.interface.entity + 0x08 * (index >> 9) + 0x10);
    if (!v1) {
        return std::nullopt;
    }
    // what?
    const u64 controller = process.Read<u64>(v1 + 120 * (index & 0x1FF));
    if (!controller) {
        return std::nullopt;
    }

    return Smoke {.controller = controller};
}

void Smoke::Disable() const {
    if (config.misc.no_smoke) {
        process.Write(controller + offsets.smoke.did_smoke_effect, true);
    }
}

void Smoke::SetColor() const {
    if (config.misc.change_smoke_color) {
        process.Write(controller + offsets.smoke.smoke_color, config.misc.smoke_color.x * 255.0f);
        process.Write(
            controller + offsets.smoke.smoke_color + 4, config.misc.smoke_color.y * 255.0f);
        process.Write(
            controller + offsets.smoke.smoke_color + 8, config.misc.smoke_color.z * 255.0f);
    }
}

void Smokes(const std::vector<Smoke> &smokes) {
    for (const auto &smoke : smokes) {
        smoke.Disable();
        smoke.SetColor();
    }
}
