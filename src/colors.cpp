#include "colors.hpp"

ImU32 Colors::ToU32(const ImVec4 &color) {
    return IM_COL32(color.x * 255.0f, color.y * 255.0f, color.z * 255.0f, 255);
}
