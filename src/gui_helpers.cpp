#include "gui_helpers.hpp"

#include <imgui_internal.h>

#include <glm/glm.hpp>

#include "globals.hpp"
#include "style.hpp"

ImU32 HealthColor(const i32 health) {
    // smooth gradient from 100 (green) over 50 (yellow) to 0 (red)
    const i32 clamped_health = glm::clamp(health, 0, 100);

    u8 r, g;

    if (clamped_health <= 50) {
        const f32 factor = static_cast<f32>(clamped_health) / 50.0f;
        r = 255;
        g = static_cast<u8>(255.0f * factor);
    } else {
        const f32 factor = static_cast<f32>(clamped_health - 50) / 50.0f;
        r = static_cast<u8>(255.0f * (1.0f - factor));
        g = 255;
    }

    return IM_COL32(r, g, 0, 255);
}

constexpr ImU32 black = 0xFF000000;
void OutlineText(
    ImDrawList *draw_list, ImFont *font, const f32 size, const ImVec2 position, const ImU32 color,
    const char *text) {
    draw_list->AddText(font, size, ImVec2 {position.x - 1, position.y}, black, text);
    draw_list->AddText(font, size, ImVec2 {position.x + 1, position.y}, black, text);
    draw_list->AddText(font, size, ImVec2 {position.x, position.y - 1}, black, text);
    draw_list->AddText(font, size, ImVec2 {position.x, position.y + 1}, black, text);
    draw_list->AddText(font, size, position, color, text);
}

void OutlineText(
    ImDrawList *draw_list, const ImVec2 position, const ImU32 color, const char *text) {
    draw_list->AddText(ImVec2 {position.x - 1, position.y}, black, text);
    draw_list->AddText(ImVec2 {position.x + 1, position.y}, black, text);
    draw_list->AddText(ImVec2 {position.x, position.y - 1}, black, text);
    draw_list->AddText(ImVec2 {position.x, position.y + 1}, black, text);
    draw_list->AddText(position, color, text);
}

void PushButtonStyle(const ImVec4 color) {
    ImGui::PushStyleColor(ImGuiCol_Text, ImVec4(0.12f, 0.12f, 0.16f, 1.0f));
    ImGui::PushStyleColor(ImGuiCol_Button, ImVec4(color.x, color.y, color.z, 0.6f));
    ImGui::PushStyleColor(ImGuiCol_ButtonActive, color);
    ImGui::PushStyleColor(ImGuiCol_ButtonHovered, color);
}

void PopButtonStyle() { ImGui::PopStyleColor(4); }

void ColorButton(const char *name, const ImVec4 color) {
    PushButtonStyle(color);
    if (ImGui::Button(name)) {
        SetAccentColor(color);
        config.accent_color = color;
    }
    PopButtonStyle();
}

void Title(const char *title) {
    ImGui::PushStyleColor(ImGuiCol_Text, Colors::SUBTEXT);
    ImGui::PushStyleColor(ImGuiCol_Separator, Colors::SUBTEXT);
    ImGui::Text("%s", title);
    ImGui::SeparatorEx(ImGuiSeparatorFlags_Horizontal, 2.0f);
    ImGui::PopStyleColor(2);
}

void BeginTitle() {
    ImGui::PushStyleColor(ImGuiCol_Text, Colors::SUBTEXT);
    ImGui::PushStyleColor(ImGuiCol_Separator, Colors::SUBTEXT);
}

void EndTitle() {
    ImGui::SeparatorEx(ImGuiSeparatorFlags_Horizontal, 2.0f);
    ImGui::PopStyleColor(2);
}

bool SidebarButton(const char *text, const ImVec2 &size, const bool active) {
    ImGui::PushStyleColor(ImGuiCol_Button, Colors::TRANSPARENT);
    if (active) {
        ImGui::PushStyleColor(ImGuiCol_Button, Colors::HIGHLIGHT);
        ImGui::PushStyleColor(ImGuiCol_Text, GetAccentColor());
    }
    bool pressed = ImGui::Button(text, size);
    if (active) {
        ImGui::PopStyleColor(2);
    }
    ImGui::PopStyleColor();
    return pressed;
}

bool TopBarButton(const char *text, const ImVec2 &size, const bool active) {
    ImGui::PushStyleColor(ImGuiCol_Button, Colors::TRANSPARENT);
    if (active) {
        ImGui::PushStyleColor(ImGuiCol_Button, Colors::HIGHLIGHT);
        ImGui::PushStyleColor(ImGuiCol_Text, GetAccentColor());
    }
    bool pressed = ImGui::Button(text, size);
    if (active) {
        ImGui::PopStyleColor(2);
    }
    ImGui::PopStyleColor();
    return pressed;
}

void Spacer() { ImGui::Dummy({1.0f, 8.0f}); }
