#pragma once

#include <imgui.h>

#include <mithril/types.hpp>

ImVec4 &GetAccentColor();
void SetAccentColor(const ImVec4 &color);
void Style();
void SetScale(f32 scale);
