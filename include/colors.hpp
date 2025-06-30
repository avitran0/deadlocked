#pragma once

#include <imgui.h>

namespace Colors {
    constexpr ImVec4 BACKDROP {0.094f, 0.094f, 0.125f, 1.0f};
    constexpr ImVec4 BASE {0.118f, 0.118f, 0.157f, 1.0f};
    constexpr ImVec4 HIGHLIGHT {0.196f, 0.196f, 0.275f, 1.0f};
    constexpr ImVec4 HIGHLIGHTER {0.28f, 0.28f, 0.37f, 1.0f};
    constexpr ImVec4 SUBTEXT {0.5f, 0.5f, 0.5f, 1.0f};
    constexpr ImVec4 TEXT {1.0f, 1.0f, 1.0f, 1.0f};

    constexpr ImVec4 RED {1.0f, 0.4f, 0.4f, 1.0f};
    constexpr ImVec4 ORANGE {1.0f, 0.55f, 0.35f, 1.0f};
    constexpr ImVec4 YELLOW {1.0f, 0.8f, 0.47f, 1.0f};
    constexpr ImVec4 GREEN {0.63f, 1.0f, 0.51f, 1.0f};
    constexpr ImVec4 CYAN {0.31f, 0.78f, 0.78f, 1.0f};
    constexpr ImVec4 BLUE {0.4f, 0.6f, 1.0f, 1.0f};
    constexpr ImVec4 PURPLE {0.7f, 0.47f, 1.0f, 1.0f};

    constexpr ImVec4 TRANSPARENT {0.0f, 0.0f, 0.0f, 0.0f};

    ImU32 ToU32(const ImVec4 &color);
}  // namespace Colors
