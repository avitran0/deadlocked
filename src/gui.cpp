#include "gui.hpp"

#include <SDL3/SDL.h>
#include <SDL3/SDL_error.h>
#include <SDL3/SDL_opengl.h>
#include <SDL3/SDL_video.h>
#include <imgui.h>
#include <imgui_impl_opengl3.h>
#include <imgui_impl_sdl3.h>

#include <chrono>
#include <cmath>
#include <mithril/logging.hpp>
#include <mithril/numbers.hpp>
#include <mithril/types.hpp>
#include <string>
#include <thread>

#include "colors.hpp"
#include "config.hpp"
#include "cs2/cs2.hpp"
#include "fonts/fira_sans_regular.hpp"
#include "fonts/icons.hpp"
#include "fonts/material_icons.hpp"
#include "globals.hpp"
#include "imgui_internal.h"
#include "math.hpp"
#include "mouse.hpp"
#include "style.hpp"

enum class Tab {
    Aimbot,
    Players,
    Hud,
    Unsafe,
    Misc,
};

Tab active_tab = Tab::Aimbot;

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

bool SidebarButton(const char *text, const ImVec2 size, const bool active) {
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

struct sizes {
    f32 scale;
    f32 spacing;
    f32 sidebar_width;
    f32 sidebar_button_height;
    ImVec2 sidebar_button_size;
    f32 top_bar_height;

    f32 combo_width;
    f32 drag_width;

    sizes(f32 scale, f32 spacing)
        : scale(scale),
          spacing(spacing),
          sidebar_width(150.0f * scale),
          sidebar_button_height(32.0f * scale),
          sidebar_button_size({sidebar_width, sidebar_button_height}),
          top_bar_height(50.0f * scale),
          combo_width(150.0f * scale),
          drag_width(100.0f * scale) {}
};

void Gui() {
    SDL_SetHint(SDL_HINT_VIDEO_DRIVER, "x11");

    if (!SDL_Init(SDL_INIT_VIDEO)) {
        logging::Error("sdl3 initialization failed, exiting");
        exit(1);
    }

    SDL_GL_SetAttribute(SDL_GL_CONTEXT_FLAGS, 0);
    SDL_GL_SetAttribute(SDL_GL_CONTEXT_PROFILE_MASK, SDL_GL_CONTEXT_PROFILE_CORE);
    SDL_GL_SetAttribute(SDL_GL_CONTEXT_MAJOR_VERSION, 3);
    SDL_GL_SetAttribute(SDL_GL_CONTEXT_MINOR_VERSION, 0);
    SDL_GL_SetAttribute(SDL_GL_DOUBLEBUFFER, 1);
    SDL_GL_SetAttribute(SDL_GL_DEPTH_SIZE, 24);

    // get monitor sizes
    i32 count {0};
    i32 minX {0}, minY {0}, maxX {0}, maxY {0};
    SDL_DisplayID *displays = SDL_GetDisplays(&count);
    SDL_Rect bounds {};
    for (i32 i = 0; i < count; i++) {
        SDL_GetDisplayBounds(displays[i], &bounds);

        if (i == 0) {
            minX = bounds.x;
            minY = bounds.y;
            maxX = bounds.x + bounds.w;
            maxY = bounds.y + bounds.h;
        } else {
            if (bounds.x < minX) {
                minX = bounds.x;
            }
            if (bounds.y < minY) {
                minY = bounds.y;
            }
            if (bounds.x + bounds.w > maxX) {
                maxX = bounds.x + bounds.w;
            }
            if (bounds.y + bounds.h > maxY) {
                maxY = bounds.y + bounds.h;
            }
        }
    }

    logging::Info("screen top left corner at: {} x {} px", minX, minY);
    logging::Info("screen resolution: {} x {} px", maxX - minX, maxY - minY);

    IMGUI_CHECKVERSION();
    ImGuiContext *gui_ctx = ImGui::CreateContext();
    ImGuiContext *overlay_ctx = ImGui::CreateContext();

    f32 scale;
    if (misc_info.gui_scale > 0.0f) {
        scale = misc_info.gui_scale;
    } else {
        const SDL_DisplayID display = SDL_GetPrimaryDisplay();
        scale = SDL_GetDisplayContentScale(display);
        logging::Info("detected display scale: {}", scale);
    }

    constexpr i32 width = 900;
    constexpr i32 height = 540;
    // gui window
    SDL_Window *gui_window = SDL_CreateWindow(
        "deadlocked", static_cast<i32>(width * scale), static_cast<i32>(height * scale),
        SDL_WINDOW_OPENGL | SDL_WINDOW_RESIZABLE | SDL_WINDOW_HIGH_PIXEL_DENSITY);
    if (!gui_window) {
        logging::Error("could not create gui window");
        logging::Error(SDL_GetError());
        return;
    }
    SDL_SetWindowPosition(gui_window, SDL_WINDOWPOS_CENTERED, SDL_WINDOWPOS_CENTERED);
    SDL_GLContext gui_gl = SDL_GL_CreateContext(gui_window);
    if (!gui_gl) {
        logging::Error("failed to initialize opengl context for gui window");
        logging::Error(SDL_GetError());
        return;
    }
    SDL_GL_MakeCurrent(gui_window, gui_gl);
    SDL_GL_SetSwapInterval(0);

    SDL_Window *temp = SDL_CreateWindow("deadlocked", 1, 1, SDL_WINDOW_BORDERLESS);
    if (!temp) {
        logging::Error("could not create overlay window");
        logging::Error(SDL_GetError());
        return;
    }
    SDL_SetWindowPosition(temp, minX, minY);

    // overlay window
    glm::ivec2 overlay_size;
    // make window not visible when flag is set
    if (flags.no_visuals) {
        overlay_size = glm::ivec2 {1};
    } else {
        overlay_size.x = maxX - minX;
        overlay_size.y = maxY - minY;
    }

    SDL_Window *overlay = SDL_CreatePopupWindow(
        temp, 0, 0, overlay_size.x, overlay_size.y,
        SDL_WINDOW_ALWAYS_ON_TOP | SDL_WINDOW_BORDERLESS | SDL_WINDOW_NOT_FOCUSABLE |
            SDL_WINDOW_OPENGL | SDL_WINDOW_TOOLTIP | SDL_WINDOW_TRANSPARENT);
    if (!overlay) {
        logging::Error("could not create overlay window");
        logging::Error(SDL_GetError());
        return;
    }

    // inherits position from parent window
    SDL_GLContext overlay_gl = SDL_GL_CreateContext(overlay);
    if (!overlay_gl) {
        logging::Error("failed to initialize opengl context for overlay window");
        logging::Error(SDL_GetError());
        return;
    }
    SDL_GL_MakeCurrent(overlay, overlay_gl);
    SDL_GL_SetSwapInterval(0);

    SDL_ShowWindow(overlay);
    SDL_ShowWindow(gui_window);

    ImGui::SetCurrentContext(gui_ctx);
    Style();
    SetScale(scale);
    SetAccentColor(config.accent_color);

    ImFontConfig font_config;
    font_config.MergeMode = true;
    font_config.PixelSnapH = true;
    font_config.GlyphOffset.y = 3.0f;
    constexpr ImWchar icon_ranges[] = {ICON_MIN_MD, ICON_MAX_16_MD, 0};

    ImGuiIO &gui_io = ImGui::GetIO();
    gui_io.IniFilename = nullptr;
    gui_io.Fonts->AddFontFromMemoryTTF(FiraSansRegular_ttf, FiraSansRegular_ttf_len, 20.0f * scale);
    gui_io.Fonts->AddFontFromMemoryTTF(
        MaterialIcons_otf, MaterialIcons_otf_len, 16.0f * scale, &font_config, icon_ranges);

    ImGui_ImplSDL3_InitForOpenGL(gui_window, gui_gl);
    ImGui_ImplOpenGL3_Init("#version 130");

    ImGui::SetCurrentContext(overlay_ctx);

    ImGuiIO &overlay_io = ImGui::GetIO();
    overlay_io.IniFilename = nullptr;
    overlay_io.Fonts->AddFontFromMemoryTTF(
        FiraSansRegular_ttf, FiraSansRegular_ttf_len, 20.0f * scale);
    overlay_io.Fonts->AddFontFromMemoryTTF(
        MaterialIcons_otf, MaterialIcons_otf_len, 16.0f * scale, &font_config, icon_ranges);

    ImGui_ImplSDL3_InitForOpenGL(overlay, overlay_gl);
    ImGui_ImplOpenGL3_Init("#version 130");

    std::thread cs2(CS2);

    const sizes sizes(scale, ImGui::GetStyle().ItemSpacing.x * 2.0f);

    bool should_close = false;
    auto save_timer = std::chrono::steady_clock::now();
    while (!should_close) {
        const auto start_time = std::chrono::steady_clock::now();

        if (!MouseValid()) {
            MouseInit();
        }

        SDL_GL_MakeCurrent(gui_window, gui_gl);
        ImGui::SetCurrentContext(gui_ctx);

        SDL_Event event;
        while (SDL_PollEvent(&event)) {
            ImGui_ImplSDL3_ProcessEvent(&event);
            if (event.type == SDL_EVENT_QUIT) should_close = true;
            if (event.type == SDL_EVENT_WINDOW_CLOSE_REQUESTED) should_close = true;
        }

        // gui
        glm::ivec2 gui_vp_size;
        ImGui_ImplOpenGL3_NewFrame();
        ImGui_ImplSDL3_NewFrame();
        SDL_GetWindowSize(gui_window, &gui_vp_size.x, &gui_vp_size.y);
        ImGui::NewFrame();
        ImGui::PushStyleColor(ImGuiCol_WindowBg, Colors::BASE);
        ImGui::Begin(
            "deadlocked", nullptr,
            ImGuiWindowFlags_NoResize | ImGuiWindowFlags_NoDecoration | ImGuiWindowFlags_NoMove |
                ImGuiWindowFlags_NoScrollbar);
        ImGui::PopStyleColor();
        ImGui::SetWindowSize(
            ImVec2 {static_cast<f32>(gui_vp_size.x), static_cast<f32>(gui_vp_size.y)});
        ImGui::SetWindowPos(ImVec2 {0.0f, 0.0f});

        // sidebar
        ImGui::BeginChild(
            "Sidebar", {sizes.sidebar_width, static_cast<f32>(gui_vp_size.y - 24)}, 0,
            ImGuiWindowFlags_NoResize | ImGuiWindowFlags_NoMove);

        ImGui::SetCursorPos({16.0f, 12.0f});
        ImGui::Text(ICON_MD_CYCLONE " deadlocked");
        ImGui::SeparatorEx(ImGuiSeparatorFlags_Horizontal, 2.0f);
        ImGui::PushStyleVar(ImGuiStyleVar_ButtonTextAlign, {0.1f, 0.5f});
        ImGui::PushStyleVar(ImGuiStyleVar_FrameRounding, 0.0f);
        if (SidebarButton(
                ICON_MD_MOUSE " Aimbot", sizes.sidebar_button_size, active_tab == Tab::Aimbot)) {
            active_tab = Tab::Aimbot;
        }
        if (SidebarButton(
                ICON_MD_GROUP " Players", sizes.sidebar_button_size, active_tab == Tab::Players)) {
            active_tab = Tab::Players;
        }
        if (SidebarButton(
                ICON_MD_MONITOR " HUD", sizes.sidebar_button_size, active_tab == Tab::Hud)) {
            active_tab = Tab::Hud;
        }
        if (SidebarButton(
                ICON_MD_ERROR_OUTLINE " Unsafe", sizes.sidebar_button_size,
                active_tab == Tab::Unsafe)) {
            active_tab = Tab::Unsafe;
        }
        if (SidebarButton(
                ICON_MD_APPS " Misc", sizes.sidebar_button_size, active_tab == Tab::Misc)) {
            active_tab = Tab::Misc;
        }
        ImGui::PopStyleVar(2);

        ImGui::EndChild();

        // top bar
        ImGui::SetCursorPos({sizes.sidebar_width + sizes.spacing + 12.0f, 12.0f});
        const ImVec2 available_top = ImGui::GetContentRegionAvail();

        ImGui::BeginChild(
            "TopBar", {available_top.x - 8.0f, sizes.top_bar_height},
            ImGuiChildFlags_AlwaysUseWindowPadding,
            ImGuiWindowFlags_NoResize | ImGuiWindowFlags_NoMove);
        ImGui::EndChild();

        // tabs
        config_lock.lock();

        ImGui::SetCursorPos({sizes.sidebar_width + sizes.spacing, sizes.top_bar_height + 12.0f});
        const ImVec2 available_main = ImGui::GetContentRegionAvail();

        const ImVec2 col_size = {
            (available_main.x - sizes.spacing * 2.0f) / 2, available_main.y - 16.0f};
        const ImVec2 current_pos = ImGui::GetCursorPos();
        ImGui::SetNextWindowPos({current_pos.x + 10.0f, current_pos.y + 15.0f});
        if (active_tab == Tab::Aimbot) {
            ImGui::BeginChild(
                "Aimbot", col_size, ImGuiChildFlags_AlwaysUseWindowPadding,
                ImGuiWindowFlags_NoResize | ImGuiWindowFlags_NoMove);

            Title("Aimbot");

            ImGui::Checkbox("Enable", &config.aimbot.enabled);

            ImGui::SetNextItemWidth(sizes.combo_width);
            if (ImGui::BeginCombo("Hotkey", key_code_names.at(config.aimbot.hotkey))) {
                for (const auto &[key, name] : key_code_names) {
                    const bool is_selected = key == config.aimbot.hotkey;
                    if (ImGui::Selectable(name, is_selected)) {
                        config.aimbot.hotkey = key;
                    }
                    if (is_selected) {
                        ImGui::SetItemDefaultFocus();
                    }
                }
                ImGui::EndCombo();
            }

            ImGui::SetNextItemWidth(sizes.drag_width);
            ImGui::DragInt("Start Bullet", &config.aimbot.start_bullet, 0.05f, 0, 10);
            if (ImGui::IsItemHovered()) {
                ImGui::SetItemTooltip(
                    "Which bullet to start aiming on, set to 0 to always be active.\nThis does not "
                    "override the hotkey.");
            }

            ImGui::SetNextItemWidth(sizes.drag_width);
            ImGui::DragFloat(
                "FOV", &config.aimbot.fov, 0.2f, 0.1f, 360.0f, "%.1f°",
                ImGuiSliderFlags_Logarithmic);

            ImGui::Checkbox("Multibone", &config.aimbot.multibone);

            ImGui::Checkbox("Aim Lock", &config.aimbot.aim_lock);
            if (!config.aimbot.aim_lock) {
                ImGui::SetNextItemWidth(sizes.drag_width);
                ImGui::DragFloat("Smooth", &config.aimbot.smooth, 0.02f, 0.0f, 10.0f, "%.1f");
            }

            Spacer();
            Title("Checks");
            ImGui::Checkbox("Visibility Check", &config.aimbot.visibility_check);
            if (ImGui::IsItemHovered()) {
                ImGui::SetItemTooltip(
                    "This is currently slow, as it\nuses the same data as the in-game-radar.");
            }
            ImGui::Checkbox("Flash Check", &config.aimbot.flash_check);

            ImGui::EndChild();

            ImGui::SameLine(0, sizes.spacing);
            ImGui::BeginChild(
                "Triggerbot", col_size, ImGuiChildFlags_AlwaysUseWindowPadding,
                ImGuiWindowFlags_NoResize | ImGuiWindowFlags_NoMove);

            Title("Triggerbot");

            ImGui::Checkbox("Enable", &config.triggerbot.enabled);

            ImGui::SetNextItemWidth(sizes.combo_width);
            if (ImGui::BeginCombo("Hotkey", key_code_names.at(config.triggerbot.hotkey))) {
                for (const auto &[key, name] : key_code_names) {
                    bool is_selected = key == config.triggerbot.hotkey;
                    if (ImGui::Selectable(name, is_selected)) {
                        config.triggerbot.hotkey = key;
                    }
                    if (is_selected) {
                        ImGui::SetItemDefaultFocus();
                    }
                }
                ImGui::EndCombo();
            }

            ImGui::SetNextItemWidth(2.0f * sizes.drag_width);
            ImGui::DragIntRange2(
                "Delay", &config.triggerbot.delay_min, &config.triggerbot.delay_max, 0.5f, 0, 1000,
                "%d ms", nullptr, ImGuiSliderFlags_AlwaysClamp);

            ImGui::Checkbox("Toggle Mode", &config.triggerbot.toggle_mode);
            if (ImGui::IsItemHovered()) {
                ImGui::SetItemTooltip(
                    "Choose whether to hold the Triggerbot button\nor whether it should toggle on "
                    "and off.");
            }

            Spacer();
            Title("Checks");

            ImGui::Checkbox("Visibility Check", &config.triggerbot.visibility_check);
            if (ImGui::IsItemHovered()) {
                ImGui::SetItemTooltip(
                    "This is currently slow, as it\nuses the same data as the in-game-radar.");
            }
            ImGui::Checkbox("Flash Check", &config.triggerbot.flash_check);
            ImGui::Checkbox("Scope Check", &config.triggerbot.scope_check);
            ImGui::Checkbox("Head Only", &config.triggerbot.head_only);

            Spacer();
            Title("Recoil");
            ImGui::Checkbox("RCS", &config.aimbot.rcs);

            ImGui::EndChild();
        } else if (active_tab == Tab::Players) {
            ImGui::BeginChild(
                "Preview", col_size, ImGuiChildFlags_AlwaysUseWindowPadding,
                ImGuiWindowFlags_NoResize | ImGuiWindowFlags_NoMove);

            Title("Preview");

            ImGui::EndChild();
            ImGui::SameLine(0, sizes.spacing);

            ImGui::BeginChild(
                "Players", col_size, ImGuiChildFlags_AlwaysUseWindowPadding,
                ImGuiWindowFlags_NoResize | ImGuiWindowFlags_NoMove);

            Title("Players");

            ImGui::Checkbox("Enable", &config.visuals.enabled);

            ImGui::SetNextItemWidth(sizes.combo_width);
            if (ImGui::BeginCombo("Box", draw_style_names.at(config.visuals.draw_box))) {
                for (const auto &[style, name] : draw_style_names) {
                    const bool is_selected = style == config.visuals.draw_box;
                    if (ImGui::Selectable(name, is_selected)) {
                        config.visuals.draw_box = style;
                    }
                    if (is_selected) {
                        ImGui::SetItemDefaultFocus();
                    }
                }
                ImGui::EndCombo();
            }

            ImGui::SetNextItemWidth(sizes.combo_width);
            if (ImGui::BeginCombo("Skeleton", draw_style_names.at(config.visuals.draw_skeleton))) {
                for (const auto &[style, name] : draw_style_names) {
                    const bool is_selected = style == config.visuals.draw_skeleton;
                    if (ImGui::Selectable(name, is_selected)) {
                        config.visuals.draw_skeleton = style;
                    }
                    if (is_selected) {
                        ImGui::SetItemDefaultFocus();
                    }
                }
                ImGui::EndCombo();
            }

            Spacer();
            Title("Info");

            ImGui::Checkbox("Health Bar", &config.visuals.draw_health);
            ImGui::Checkbox("Armor Bar", &config.visuals.draw_armor);

            ImGui::Checkbox("Player Name", &config.visuals.draw_name);
            ImGui::Checkbox("Weapon Name", &config.visuals.draw_weapon);

            ImGui::Checkbox("Player Tags", &config.visuals.draw_tags);
            if (ImGui::IsItemHovered()) {
                ImGui::SetItemTooltip("Shows whether the player has\nhelmet, defuser, or bomb.");
            }

            Spacer();
            Title("Colors");

            if (config.visuals.draw_box == DrawStyle::Color) {
                ImGui::ColorEdit3(
                    "Box Color", &config.visuals.box_color.x, ImGuiColorEditFlags_NoInputs);
            }
            if (config.visuals.draw_skeleton == DrawStyle::Color) {
                ImGui::ColorEdit3(
                    "Skeleton Color", &config.visuals.skeleton_color.x,
                    ImGuiColorEditFlags_NoInputs);
            }

            ImGui::EndChild();
        } else if (active_tab == Tab::Hud) {
            ImGui::BeginChild(
                "HUD", col_size, ImGuiChildFlags_AlwaysUseWindowPadding,
                ImGuiWindowFlags_NoResize | ImGuiWindowFlags_NoMove);

            Title("HUD");

            ImGui::Checkbox("FOV Circle", &config.aimbot.fov_circle);
            ImGui::Checkbox("Spectator List", &config.visuals.spectator_list);

            ImGui::Checkbox("Show Dropped Weapons", &config.visuals.dropped_weapons);
            ImGui::Checkbox("Sniper Crosshair", &config.visuals.sniper_crosshair);

            ImGui::SetNextItemWidth(sizes.combo_width);
            if (ImGui::BeginCombo(
                    "Triggerbot Indicator",
                    position_names.at(config.triggerbot.indicator_position))) {
                for (const auto &[position, name] : position_names) {
                    const bool is_selected = position == config.triggerbot.indicator_position;
                    if (ImGui::Selectable(name, is_selected)) {
                        config.triggerbot.indicator_position = position;
                    }
                    if (is_selected) {
                        ImGui::SetItemDefaultFocus();
                    }
                }
                ImGui::EndCombo();
            }

            ImGui::SetNextItemWidth(sizes.drag_width);
            ImGui::DragFloat(
                "##indicator_inset_x", &config.triggerbot.indicator_inset.x, 0.2f, 0.0f, 9999.0f,
                "x: %.1f");
            ImGui::SameLine();
            ImGui::SetNextItemWidth(sizes.drag_width);
            ImGui::DragFloat(
                "Indicator Inset", &config.triggerbot.indicator_inset.y, 1.0f, 0.0f, 9999.0f,
                "y: %.1f");

            Spacer();
            Title("Colors");

            ImGui::ColorEdit3(
                "Text Color", &config.visuals.text_color.x, ImGuiColorEditFlags_NoInputs);
            if (config.visuals.sniper_crosshair) {
                ImGui::ColorEdit3(
                    "Crosshair Color", &config.visuals.crosshair_color.x,
                    ImGuiColorEditFlags_NoInputs);
            }

            ImGui::EndChild();
            ImGui::SameLine(0, sizes.spacing);

            ImGui::BeginChild(
                "Advanced", col_size, ImGuiChildFlags_AlwaysUseWindowPadding,
                ImGuiWindowFlags_NoResize | ImGuiWindowFlags_NoMove);

            Title("Advanced");

            ImGui::SetNextItemWidth(sizes.drag_width);
            ImGui::DragFloat("Font Size", &config.visuals.font_size, 0.02f, 1.0f, 50.0f, "%.1f");
            ImGui::SetNextItemWidth(sizes.drag_width);
            ImGui::DragFloat("Line Width", &config.visuals.line_width, 0.01f, 0.2f, 3.0f, "%.1f");

            ImGui::SetNextItemWidth(sizes.drag_width);
            ImGui::DragInt("Overlay FPS", &config.visuals.overlay_fps, 0.2f, 60, 240);
            ImGui::Checkbox("Debug Overlay", &config.visuals.debug_window);

            ImGui::EndChild();
        } else if (active_tab == Tab::Unsafe) {
            ImGui::BeginChild(
                "Unsafe", col_size, ImGuiChildFlags_AlwaysUseWindowPadding,
                ImGuiWindowFlags_NoResize | ImGuiWindowFlags_NoMove);

            Title("Unsafe");

            ImGui::Checkbox("No Flash", &config.misc.no_flash);
            if (config.misc.no_flash) {
                ImGui::DragFloat(
                    "Max Flash Alpha", &config.misc.max_flash_alpha, 0.2f, 0.0f, 255.0f, "%.0f");
            }

            ImGui::Checkbox("FOV Changer", &config.misc.fov_changer);
            if (config.misc.fov_changer) {
                ImGui::DragInt("Desired FOV", &config.misc.desired_fov, 0.2f, 1, 179);
                if (ImGui::Button("Reset FOV")) {
                    config.misc.desired_fov = DEFAULT_FOV;
                }
            }

            ImGui::EndChild();
        } else if (active_tab == Tab::Misc) {
            ImGui::BeginChild(
                "Misc", col_size, ImGuiChildFlags_AlwaysUseWindowPadding,
                ImGuiWindowFlags_NoResize | ImGuiWindowFlags_NoMove);

            Title("Misc");

            if (ImGui::Button("Reset Config")) {
                ResetConfig();
            }

            if (ImGui::Button("Report Issue")) {
                std::system("xdg-open https://github.com/avitran0/deadlocked");
            }

            ImGui::Text("Accent Color");

            ColorButton("Red", Colors::RED);
            ImGui::SameLine();
            ColorButton("Orange", Colors::ORANGE);
            ImGui::SameLine();
            ColorButton("Yellow", Colors::YELLOW);

            ColorButton("Green", Colors::GREEN);
            ImGui::SameLine();
            ColorButton("Cyan", Colors::CYAN);
            ImGui::SameLine();
            ColorButton("Blue", Colors::BLUE);
            ImGui::SameLine();
            ColorButton("Purple", Colors::PURPLE);

            ImGui::EndChild();
        }

        ImDrawList *gui_draw_list = ImGui::GetForegroundDrawList();
        std::string gui_fps =
            VERSION " | FPS: " + std::to_string(static_cast<i32>(gui_io.Framerate));
        const ImVec2 text_size = ImGui::CalcTextSize(gui_fps.c_str());
        const ImVec2 gui_window_size = ImGui::GetWindowSize();
        gui_draw_list->AddText(
            ImVec2 {24.0f, gui_window_size.y - text_size.y - 20.0f}, 0xFFFFFFFF, gui_fps.c_str());

        ImGui::End();

        ImGui::Render();
        glViewport(0, 0, gui_vp_size.x, gui_vp_size.y);
        glClearColor(0.1176470592617989f, 0.1176470592617989f, 0.1568627506494522f, 1.0f);
        glClear(GL_COLOR_BUFFER_BIT);
        ImGui_ImplOpenGL3_RenderDrawData(ImGui::GetDrawData());

        SDL_GL_SwapWindow(gui_window);

        // overlay
        SDL_GL_MakeCurrent(overlay, overlay_gl);
        SDL_SetWindowPosition(temp, 0, 0);
        SDL_SetWindowPosition(overlay, minX, minY);
        ImGui::SetCurrentContext(overlay_ctx);
        ImGui_ImplOpenGL3_NewFrame();
        ImGui_ImplSDL3_NewFrame();
        ImGui::NewFrame();

        ImGui::Begin(
            "overlay", nullptr,
            ImGuiWindowFlags_NoBackground | ImGuiWindowFlags_NoDecoration |
                ImGuiWindowFlags_NoInputs | ImGuiWindowFlags_NoResize | ImGuiWindowFlags_NoMove);
        ImGui::SetWindowPos(ImVec2 {static_cast<f32>(minX), static_cast<f32>(minY)});
        ImGui::SetWindowSize(ImVec2 {static_cast<f32>(maxX - minX), static_cast<f32>(maxY - minY)});
        ImDrawList *overlay_draw_list = ImGui::GetBackgroundDrawList();

        const ImU32 text_color = IM_COL32(
            config.visuals.text_color.x * 255, config.visuals.text_color.y * 255,
            config.visuals.text_color.z * 255, 255);

        if (config.visuals.debug_window) {
            // frame
            overlay_draw_list->AddRect(
                ImVec2 {static_cast<f32>(minX), static_cast<f32>(minY)},
                ImVec2 {static_cast<f32>(maxX - minX), static_cast<f32>(maxY - minY)}, 0xFFFFFFFF,
                0.0f, 0, 8.0f);

            // cross
            overlay_draw_list->AddLine(
                ImVec2 {static_cast<f32>(minX), static_cast<f32>(minY)},
                ImVec2 {static_cast<f32>(maxX - minX), static_cast<f32>(maxY - minY)}, 0xFFFFFFFF,
                4.0f);
            overlay_draw_list->AddLine(
                ImVec2 {static_cast<f32>(minX), static_cast<f32>(maxY - minY)},
                ImVec2 {static_cast<f32>(maxX - minX), static_cast<f32>(minY)}, 0xFFFFFFFF, 4.0f);
        }
        if (config.visuals.enabled) {
            vinfo_lock.lock();
            f32 spectator_offset = 4.0f + 20.0f * scale;
            for (const std::string &player : misc_info.spectators) {
                OutlineText(
                    overlay_draw_list, ImVec2 {spectator_offset, 4.0f}, 0xFFFFFFFF, player.c_str());
                spectator_offset += 20.0f * scale;
            }

            for (const PlayerInfo &player : player_info) {
                if (!misc_info.is_ffa && player.team == local_player.team) {
                    continue;
                }

                const ImU32 health_color = HealthColor(player.health);

                if (config.visuals.draw_skeleton != DrawStyle::None) {
                    ImU32 color;
                    if (config.visuals.draw_skeleton == DrawStyle::Color) {
                        color = IM_COL32(
                            config.visuals.skeleton_color.x * 255,
                            config.visuals.skeleton_color.y * 255,
                            config.visuals.skeleton_color.z * 255, 255);
                    } else {
                        color = health_color;
                    }
                    for (const auto &[first, second] : player.bones) {
                        const auto bone1 = WorldToScreen(first);
                        const auto bone2 = WorldToScreen(second);
                        if (bone1 && bone2) {
                            const ImVec2 start {bone1->x, bone1->y};
                            const ImVec2 end {bone2->x, bone2->y};
                            overlay_draw_list->AddLine(
                                start, end, color, config.visuals.line_width);
                        }
                    }
                }

                const auto bottom_opt = WorldToScreen(player.position);
                const auto top_opt = WorldToScreen(player.head + glm::vec3(0.0f, 0.0f, 8.0f));

                if (!bottom_opt || !top_opt) {
                    continue;
                }

                const ImVec2 bottom {bottom_opt->x, bottom_opt->y};
                const ImVec2 top {bottom.x, bottom.y + (top_opt->y - bottom.y)};

                const f32 box_height = bottom.y - top.y;
                const f32 box_width = box_height / 2.0f;
                const f32 half_width = box_width / 2.0f;
                f32 font_size = config.visuals.font_size * scale;
                if (font_size > 20.0f * scale) {
                    font_size = 20.0f * scale;
                }
                ImFont *font = overlay_io.Fonts->Fonts[0];

                const ImVec2 bottom_left {bottom.x - half_width, bottom.y};
                const ImVec2 bottom_right {bottom.x + half_width, bottom.y};
                const ImVec2 top_left {top.x - half_width, top.y};
                const ImVec2 top_right {top.x + half_width, top.y};

                if (config.visuals.draw_box != DrawStyle::None) {
                    // four corners, each a quarter of the width/height
                    // convert imvec4 to imu32
                    ImU32 color;
                    if (config.visuals.draw_box == DrawStyle::Color) {
                        color = IM_COL32(
                            config.visuals.box_color.x * 255, config.visuals.box_color.y * 255,
                            config.visuals.box_color.z * 255, 255);
                    } else {
                        color = health_color;
                    }
                    overlay_draw_list->AddLine(
                        bottom_left, ImVec2 {bottom_left.x, bottom_left.y - box_height / 4.0f},
                        color, config.visuals.line_width);
                    overlay_draw_list->AddLine(
                        bottom_left, ImVec2 {bottom_left.x + box_width / 4.0f, bottom_left.y},
                        color, config.visuals.line_width);
                    overlay_draw_list->AddLine(
                        bottom_right, ImVec2 {bottom_right.x, bottom_right.y - box_height / 4.0f},
                        color, config.visuals.line_width);
                    overlay_draw_list->AddLine(
                        bottom_right, ImVec2 {bottom_right.x - box_width / 4.0f, bottom_right.y},
                        color, config.visuals.line_width);
                    overlay_draw_list->AddLine(
                        top_left, ImVec2 {top_left.x, top_left.y + box_height / 4.0f}, color,
                        config.visuals.line_width);
                    overlay_draw_list->AddLine(
                        top_left, ImVec2 {top_left.x + box_width / 4.0f, top_left.y}, color,
                        config.visuals.line_width);
                    overlay_draw_list->AddLine(
                        top_right, ImVec2 {top_right.x, top_right.y + box_height / 4.0f}, color,
                        config.visuals.line_width);
                    overlay_draw_list->AddLine(
                        top_right, ImVec2 {top_right.x - box_width / 4.0f, top_right.y}, color,
                        config.visuals.line_width);
                }

                if (config.visuals.draw_health) {
                    const ImVec2 health_bottom {bottom_left.x - 4.0f, bottom_left.y};
                    // adjust height based on health
                    const ImVec2 health_top {
                        top_left.x - 4.0f,
                        bottom_left.y - box_height * static_cast<f32>(player.health) / 100.0f};
                    overlay_draw_list->AddLine(
                        health_bottom, health_top, health_color, config.visuals.line_width);
                }

                if (config.visuals.draw_armor) {
                    const ImVec2 armor_bottom {bottom_left.x - 8, bottom_left.y};
                    const ImVec2 armor_top {
                        top_left.x - 8,
                        bottom_left.y - box_height * static_cast<f32>(player.armor) / 100.0f};
                    overlay_draw_list->AddLine(
                        armor_bottom, armor_top,
                        IM_COL32(
                            config.visuals.armor_color.x * 255, config.visuals.armor_color.y * 255,
                            config.visuals.armor_color.z * 255, 255),
                        config.visuals.line_width);
                }

                if (config.visuals.draw_name) {
                    const ImVec2 name_text_size =
                        font->CalcTextSizeA(font_size, FLT_MAX, 0.0f, player.name.c_str());
                    const ImVec2 name_position {
                        top.x - name_text_size.x / 2.0f, top_left.y - font_size};
                    OutlineText(
                        overlay_draw_list, font, font_size, name_position, text_color,
                        player.name.c_str());
                }

                if (config.visuals.draw_weapon) {
                    const ImVec2 weapon_text_size =
                        font->CalcTextSizeA(font_size, FLT_MAX, 0.0f, player.weapon.c_str());
                    const ImVec2 weapon_position {
                        bottom.x - weapon_text_size.x / 2.0f, bottom_left.y};
                    OutlineText(
                        overlay_draw_list, font, font_size, weapon_position, text_color,
                        player.weapon.c_str());
                }

                f32 offset = font_size;

                if (config.visuals.draw_tags && player.has_helmet) {
                    const ImVec2 helmet_text_size =
                        font->CalcTextSizeA(font_size, FLT_MAX, 0.0f, "helmet");
                    const ImVec2 helmet_position {
                        bottom.x - helmet_text_size.x / 2.0f, bottom_left.y + offset};
                    offset += font_size;
                    OutlineText(
                        overlay_draw_list, font, font_size, helmet_position, text_color, "helmet");
                }

                if (config.visuals.draw_tags && player.has_defuser) {
                    const ImVec2 defuser_text_size =
                        font->CalcTextSizeA(font_size, FLT_MAX, 0.0f, "defuser");
                    const ImVec2 defuser_position {
                        bottom.x - defuser_text_size.x / 2.0f, bottom_left.y + offset};
                    offset += font_size;
                    OutlineText(
                        overlay_draw_list, font, font_size, defuser_position, text_color,
                        "defuser");
                }

                if (config.visuals.draw_tags && player.has_bomb) {
                    const ImVec2 bomb_text_size =
                        font->CalcTextSizeA(font_size, FLT_MAX, 0.0f, "bomb");
                    const ImVec2 bomb_position {
                        bottom.x - bomb_text_size.x / 2.0f, bottom_left.y + offset};
                    OutlineText(
                        overlay_draw_list, font, font_size, bomb_position, text_color, "bomb");
                }
            }

            if (config.visuals.dropped_weapons) {
                for (const auto &[name, position] : entity_info) {
                    const auto screen_position = WorldToScreen(position);
                    if (!screen_position) {
                        continue;
                    }
                    OutlineText(
                        overlay_draw_list, ImVec2 {screen_position->x, screen_position->y},
                        0xFFFFFFFF, name.c_str());
                }
            }

            // fov circle
            if (config.aimbot.fov_circle && misc_info.in_game) {
                const f32 pawn_fov =
                    config.misc.fov_changer ? static_cast<f32>(config.misc.desired_fov) : 90.0f;
                const f32 radius = tanf(config.aimbot.fov / 180.0f * numbers::pi<f32>() / 2.0f) /
                                   tanf(pawn_fov / 180.0f * numbers::pi<f32>() / 2.0f) *
                                   window_size.z / 2.0f;
                const ImVec2 center {
                    window_size.x + window_size.z / 2.0f, window_size.y + window_size.w / 2.0f};
                overlay_draw_list->AddCircle(
                    center, radius, 0xFFFFFFFF, 0, config.visuals.line_width);
            }

            // sniper crosshair
            if (config.visuals.sniper_crosshair &&
                WeaponClassFromString(misc_info.held_weapon) == WeaponClass::Sniper) {
                constexpr f32 crosshair_size = 32.0f;
                const ImVec2 center {
                    window_size.x + window_size.z / 2.0f, window_size.y + window_size.w / 2.0f};
                const ImU32 color = IM_COL32(
                    config.visuals.crosshair_color.x * 255, config.visuals.crosshair_color.y * 255,
                    config.visuals.crosshair_color.z * 255, 255);
                overlay_draw_list->AddLine(
                    ImVec2 {center.x - crosshair_size, center.y},
                    ImVec2 {center.x + crosshair_size, center.y}, color, config.visuals.line_width);
                overlay_draw_list->AddLine(
                    ImVec2 {center.x, center.y - crosshair_size},
                    ImVec2 {center.x, center.y + crosshair_size}, color, config.visuals.line_width);
            }

            vinfo_lock.unlock();
        }

        if (misc_info.triggerbot_active && misc_info.in_game) {
            const ImVec2 text_size = ImGui::CalcTextSize("Trigger Enabled");
            ImVec2 tb_position {};
            const f32 offset = 4.0f * scale;
            const ImVec2 inset = config.triggerbot.indicator_inset;
            switch (config.triggerbot.indicator_position) {
                case Position::TopLeft:
                    tb_position = {
                        window_size.x + offset + inset.x, window_size.y + offset * inset.y};
                    break;
                case Position::TopRight:
                    tb_position = {
                        window_size.x + window_size.z - text_size.x - offset - inset.x,
                        window_size.y + offset + inset.y};
                    break;
                case Position::BottomLeft:
                    tb_position = {
                        window_size.x + offset + inset.x,
                        window_size.y + window_size.w - text_size.y - offset - inset.y};
                    break;
                case Position::BottomRight:
                    tb_position = {
                        window_size.x + window_size.z - text_size.x - offset - inset.x,
                        window_size.y + window_size.w - text_size.y - offset - inset.y};
                    break;
            }
            OutlineText(overlay_draw_list, tb_position, 0xFFFFFFFF, "Trigger Enabled");
        }

        ImGui::End();
        config_lock.unlock();

        ImGui::Render();
        glm::ivec2 overlay_vp_size;
        SDL_GetWindowSize(overlay, &overlay_vp_size.x, &overlay_vp_size.y);
        glViewport(0, 0, overlay_vp_size.x, overlay_vp_size.y);
        glClearColor(0.0f, 0.0f, 0.0f, 0.0f);
        glClear(GL_COLOR_BUFFER_BIT);
        ImGui_ImplOpenGL3_RenderDrawData(ImGui::GetDrawData());

        SDL_GL_SwapWindow(overlay);

        const auto end_time = std::chrono::steady_clock::now();

        if (end_time - save_timer > save_interval) {
            SaveConfig();
            save_timer = end_time;
        }

        const auto us =
            std::chrono::duration_cast<std::chrono::microseconds>(end_time - start_time);
        const auto frame_time =
            std::chrono::microseconds(1000000 / (config.visuals.overlay_fps + 1));
        std::this_thread::sleep_for(frame_time - us);
        // glfwPollEvents();
    }

    logging::Info("shutting down...");

    config_lock.lock();
    flags.should_quit = true;
    config_lock.unlock();
    cs2.join();

    ImGui::SetCurrentContext(gui_ctx);
    ImGui_ImplOpenGL3_Shutdown();
    ImGui_ImplSDL3_Shutdown();

    ImGui::SetCurrentContext(overlay_ctx);
    ImGui_ImplOpenGL3_Shutdown();
    ImGui_ImplSDL3_Shutdown();

    // why are these pointers invalid?
    // ImGui::DestroyContext(gui_ctx);
    // ImGui::DestroyContext(overlay_ctx);

    SDL_GL_DestroyContext(gui_gl);
    SDL_GL_DestroyContext(overlay_gl);
    SDL_DestroyWindow(gui_window);
    SDL_DestroyWindow(overlay);
    SDL_Quit();

    MouseQuit();
}
