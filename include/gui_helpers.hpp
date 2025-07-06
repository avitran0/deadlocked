#pragma once

#include <imgui.h>

#include <mithril/types.hpp>

ImU32 HealthColor(const i32 health);

void OutlineText(
    ImDrawList *draw_list, ImFont *font, const f32 size, const ImVec2 position, const ImU32 color,
    const char *text);
void OutlineText(ImDrawList *draw_list, const ImVec2 position, const ImU32 color, const char *text);

void ColorButton(const char *name, const ImVec4 color);
void Title(const char *title);
void BeginTitle();
void EndTitle();
bool SidebarButton(const char *text, const ImVec2 &size, const bool active);
bool TopBarButton(const char *text, const ImVec2 &height, const bool active);
void Spacer();
