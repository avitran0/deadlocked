#include "gui.hpp"

#include <SDL3/SDL.h>
#include <SDL3/SDL_opengl.h>
#include <imgui.h>
#include <imgui_impl_opengl3.h>
#include <imgui_impl_sdl3.h>

#include <chrono>
#include <log.hpp>
#include <string>
#include <thread>

#include "SDL3/SDL_error.h"
#include "SDL3/SDL_video.h"
#include "colors.hpp"
#include "config.hpp"
#include "cs2/cs2.hpp"
#include "font.hpp"
#include "math.hpp"
#include "style.hpp"
#include "types.hpp"

extern std::mutex config_lock;
extern Config config;
extern std::mutex vinfo_lock;
extern std::vector<PlayerInfo> player_info;
extern std::vector<EntityInfo> entity_info;
extern MiscInfo misc_info;

ImU32 HealthColor(i32 health) {
    // smooth gradient from 100 (green) over 50 (yellow) to 0 (red)
    health = std::clamp(health, 0, 100);

    u8 r, g;

    if (health <= 50) {
        f32 factor = (f32)health / 50.0f;
        r = 255;
        g = (u8)(255.0f * factor);
    } else {
        f32 factor = (f32)(health - 50) / 50.0f;
        r = (u8)(255.0f * (1.0f - factor));
        g = 255;
    }

    return IM_COL32(r, g, 0, 255);
}

const ImU32 black = 0xFF000000;
void OutlineText(
    ImDrawList *draw_list, ImFont *font, f32 size, const ImVec2 position, const ImU32 color,
    const char *text) {
    draw_list->AddText(font, size, ImVec2(position.x - 1, position.y), black, text);
    draw_list->AddText(font, size, ImVec2(position.x + 1, position.y), black, text);
    draw_list->AddText(font, size, ImVec2(position.x, position.y - 1), black, text);
    draw_list->AddText(font, size, ImVec2(position.x, position.y + 1), black, text);
    draw_list->AddText(font, size, position, color, text);
}

void OutlineText(
    ImDrawList *draw_list, const ImVec2 position, const ImU32 color, const char *text) {
    draw_list->AddText(ImVec2(position.x - 1, position.y), black, text);
    draw_list->AddText(ImVec2(position.x + 1, position.y), black, text);
    draw_list->AddText(ImVec2(position.x, position.y - 1), black, text);
    draw_list->AddText(ImVec2(position.x, position.y + 1), black, text);
    draw_list->AddText(position, color, text);
}

void PushButtonStyle(const ImVec4 color) {
    ImGui::PushStyleColor(ImGuiCol_Text, ImVec4(0.12f, 0.12f, 0.16f, 1.0f));
    ImGui::PushStyleColor(ImGuiCol_Button, ImVec4(color.x, color.y, color.z, 0.6f));
    ImGui::PushStyleColor(ImGuiCol_ButtonActive, color);
    ImGui::PushStyleColor(ImGuiCol_ButtonHovered, color);
}

void PopButtonStyle() { ImGui::PopStyleColor(4); }

void Gui() {
    SDL_SetHint(SDL_HINT_VIDEO_DRIVER, "x11");

    if (!SDL_Init(SDL_INIT_VIDEO)) {
        Log(LogLevel::Error, "sdl3 initialization failed, exiting");
        exit(1);
    }

    SDL_GL_SetAttribute(SDL_GL_CONTEXT_FLAGS, 0);
    SDL_GL_SetAttribute(SDL_GL_CONTEXT_PROFILE_MASK, SDL_GL_CONTEXT_PROFILE_CORE);
    SDL_GL_SetAttribute(SDL_GL_CONTEXT_MAJOR_VERSION, 3);
    SDL_GL_SetAttribute(SDL_GL_CONTEXT_MINOR_VERSION, 0);
    SDL_GL_SetAttribute(SDL_GL_DOUBLEBUFFER, 1);
    SDL_GL_SetAttribute(SDL_GL_DEPTH_SIZE, 24);

    // get monitor sizes
    i32 count = 0;
    i32 minX, minY, maxX, maxY = 0;
    SDL_DisplayID *displays = SDL_GetDisplays(&count);
    for (i32 i = 0; i < count; i++) {
        SDL_Rect bounds;
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

    Log(LogLevel::Info, "screen top left corner at: " + std::to_string(minX) + " x " +
                            std::to_string(minY) + " px");
    Log(LogLevel::Info, "screen resolution: " + std::to_string(maxX - minX) + " x " +
                            std::to_string(maxY - minY) + " px");

    IMGUI_CHECKVERSION();
    ImGuiContext *gui_ctx = ImGui::CreateContext();
    ImGuiContext *overlay_ctx = ImGui::CreateContext();

    const i32 width = 620;
    const i32 height = 400;
    // gui window
    SDL_Window *gui_window =
        SDL_CreateWindow("deadlocked", width, height, SDL_WINDOW_OPENGL | SDL_WINDOW_RESIZABLE);
    if (!gui_window) {
        Log(LogLevel::Error, "could not create gui window");
        Log(LogLevel::Error, SDL_GetError());
        return;
    }
    SDL_SetWindowPosition(gui_window, SDL_WINDOWPOS_CENTERED, SDL_WINDOWPOS_CENTERED);
    SDL_GLContext gui_gl = SDL_GL_CreateContext(gui_window);
    if (!gui_gl) {
        Log(LogLevel::Error, "failed to initialize opengl context for gui window");
        Log(LogLevel::Error, SDL_GetError());
        return;
    }
    SDL_GL_MakeCurrent(gui_window, gui_gl);
    SDL_GL_SetSwapInterval(0);

    SDL_Window *temp = SDL_CreateWindow("deadlocked", 1, 1, SDL_WINDOW_BORDERLESS);
    if (!temp) {
        Log(LogLevel::Error, "could not create overlay window");
        Log(LogLevel::Error, SDL_GetError());
        return;
    }
    SDL_SetWindowPosition(temp, minX, minY);

    // overlay window
    i32 overlay_width, overlay_height;
    // make window not visible when flag is set
    if (flags.no_visuals) {
        overlay_width = 1;
        overlay_height = 1;
    } else {
        overlay_width = maxX - minX;
        overlay_height = maxY - minY;
    }

    SDL_Window *overlay = SDL_CreatePopupWindow(
        temp, 0, 0, overlay_width, overlay_height,
        SDL_WINDOW_ALWAYS_ON_TOP | SDL_WINDOW_BORDERLESS | SDL_WINDOW_NOT_FOCUSABLE |
            SDL_WINDOW_OPENGL | SDL_WINDOW_TOOLTIP | SDL_WINDOW_TRANSPARENT);
    if (!overlay) {
        Log(LogLevel::Error, "could not create overlay window");
        return;
    }
    // inherits position from parent window
    // SDL_SetWindowPosition(overlay, minX, minY);
    SDL_GLContext overlay_gl = SDL_GL_CreateContext(overlay);
    if (!overlay_gl) {
        Log(LogLevel::Error, "failed to initialize opengl context for overlay window");
        Log(LogLevel::Error, SDL_GetError());
        return;
    }
    SDL_GL_MakeCurrent(overlay, overlay_gl);
    SDL_GL_SetSwapInterval(0);

    SDL_SetWindowPosition(temp, 0, 0);

    SDL_ShowWindow(overlay);
    SDL_ShowWindow(gui_window);

    f32 scale;
    if (misc_info.gui_scale > 0.0f) {
        scale = misc_info.gui_scale;
    } else {
        const SDL_DisplayID display = SDL_GetPrimaryDisplay();
        scale = SDL_GetDisplayContentScale(display);
        Log(LogLevel::Info, "detected display scale: " + std::to_string(scale));
    }

    SDL_SetWindowSize(gui_window, width * scale, height * scale);

    ImGui::SetCurrentContext(gui_ctx);
    Style();
    SetScale(scale);
    SetAccentColor(config.accent_color);

    ImGuiIO &gui_io = ImGui::GetIO();
    gui_io.IniFilename = nullptr;
    gui_io.Fonts->AddFontFromMemoryTTF(font, font_len, 20.0f * scale);

    ImGui_ImplSDL3_InitForOpenGL(gui_window, gui_gl);
    ImGui_ImplOpenGL3_Init("#version 130");

    ImGui::SetCurrentContext(overlay_ctx);

    ImGuiIO &overlay_io = ImGui::GetIO();
    overlay_io.IniFilename = nullptr;
    overlay_io.Fonts->AddFontFromMemoryTTF(font, font_len, 20.0f * scale);

    ImGui_ImplSDL3_InitForOpenGL(overlay, overlay_gl);
    ImGui_ImplOpenGL3_Init("#version 130");

    std::thread cs2(CS2);

    bool should_close = false;
    auto save_timer = std::chrono::steady_clock::now();
    const std::chrono::seconds save_interval(1);
    while (!should_close) {
        auto start_time = std::chrono::steady_clock::now();

        SDL_GL_MakeCurrent(gui_window, gui_gl);
        ImGui::SetCurrentContext(gui_ctx);

        SDL_Event event;
        while (SDL_PollEvent(&event)) {
            ImGui_ImplSDL3_ProcessEvent(&event);
            if (event.type == SDL_EVENT_QUIT) should_close = true;
            if (event.type == SDL_EVENT_WINDOW_CLOSE_REQUESTED) should_close = true;
        }

        // gui
        i32 width, height;
        ImGui_ImplOpenGL3_NewFrame();
        ImGui_ImplSDL3_NewFrame();
        SDL_GetWindowSize(gui_window, &width, &height);
        ImGui::NewFrame();
        ImGui::Begin(
            "deadlocked", nullptr,
            ImGuiWindowFlags_NoResize | ImGuiWindowFlags_NoDecoration | ImGuiWindowFlags_NoMove);
        ImGui::SetWindowSize(ImVec2(width, height));
        ImGui::SetWindowPos(ImVec2(0.0f, 0.0f));

        // tabs
        config_lock.lock();
        ImGui::BeginTabBar("tabs");

        if (ImGui::BeginTabItem("Aimbot")) {
            ImVec2 available = ImGui::GetContentRegionAvail();
            ImGui::BeginChild("tab_items_aimbot", available);

            ImGui::Checkbox("Enable", &config.aimbot.enabled);

            if (ImGui::BeginCombo("Hotkey", key_code_names.at(config.aimbot.hotkey))) {
                for (const auto &[key, name] : key_code_names) {
                    bool is_selected = key == config.aimbot.hotkey;
                    if (ImGui::Selectable(name, is_selected)) {
                        config.aimbot.hotkey = key;
                    }
                    if (is_selected) {
                        ImGui::SetItemDefaultFocus();
                    }
                }
                ImGui::EndCombo();
            }

            ImGui::DragInt("Start Bullet", &config.aimbot.start_bullet, 0.05f, 0, 10);

            ImGui::Checkbox("Multibone", &config.aimbot.multibone);
            ImGui::SameLine();
            ImGui::Checkbox("FOV Circle", &config.aimbot.fov_circle);

            ImGui::Checkbox("Visibility Check", &config.aimbot.visibility_check);
            ImGui::SameLine();
            ImGui::Checkbox("Flash Check", &config.aimbot.flash_check);

            ImGui::DragFloat("FOV", &config.aimbot.fov, 0.02f, 0.1f, 360.0f, "%.1f");

            ImGui::Checkbox("Aim Lock", &config.aimbot.aim_lock);
            ImGui::DragFloat("Smooth", &config.aimbot.smooth, 0.02f, 1.0f, 10.0f, "%.1f");

            ImGui::Checkbox("RCS", &config.aimbot.rcs);

            ImGui::EndChild();
            ImGui::EndTabItem();
        }

        if (ImGui::BeginTabItem("Triggerbot")) {
            ImVec2 available = ImGui::GetContentRegionAvail();
            ImGui::BeginChild("tab_items_triggerbot", available);

            ImGui::Checkbox("Enable", &config.triggerbot.enabled);

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

            ImGui::DragIntRange2(
                "Delay", &config.triggerbot.delay_min, &config.triggerbot.delay_max, 0.2f, 0, 1000,
                "%d", nullptr, ImGuiSliderFlags_AlwaysClamp);

            ImGui::Checkbox("Visibility Check", &config.triggerbot.visibility_check);

            ImGui::Checkbox("Flash Check", &config.triggerbot.flash_check);

            ImGui::Checkbox("Scope Check", &config.triggerbot.scope_check);

            ImGui::EndChild();
            ImGui::EndTabItem();
        }

        if (ImGui::BeginTabItem("Visuals")) {
            ImVec2 available = ImGui::GetContentRegionAvail();
            ImGui::BeginChild("tab_items_visuals", available);

            ImGui::Checkbox("Enable", &config.visuals.enabled);

            ImGui::Text("Draw Box");
            ImGui::SameLine();
            ImGui::PushID("draw_box");
            if (ImGui::RadioButton("None", config.visuals.draw_box == DrawStyle::DrawNone)) {
                config.visuals.draw_box = DrawStyle::DrawNone;
            }
            ImGui::SameLine();
            if (ImGui::RadioButton("Color", config.visuals.draw_box == DrawStyle::DrawColor)) {
                config.visuals.draw_box = DrawStyle::DrawColor;
            }
            ImGui::SameLine();
            if (ImGui::RadioButton("Health", config.visuals.draw_box == DrawStyle::DrawHealth)) {
                config.visuals.draw_box = DrawStyle::DrawHealth;
            }
            ImGui::PopID();

            ImGui::Text("Draw Skeleton");
            ImGui::SameLine();
            ImGui::PushID("draw_skeleton");
            if (ImGui::RadioButton("None", config.visuals.draw_skeleton == DrawStyle::DrawNone)) {
                config.visuals.draw_skeleton = DrawStyle::DrawNone;
            }
            ImGui::SameLine();
            if (ImGui::RadioButton("Color", config.visuals.draw_skeleton == DrawStyle::DrawColor)) {
                config.visuals.draw_skeleton = DrawStyle::DrawColor;
            }
            ImGui::SameLine();
            if (ImGui::RadioButton(
                    "Health", config.visuals.draw_skeleton == DrawStyle::DrawHealth)) {
                config.visuals.draw_skeleton = DrawStyle::DrawHealth;
            }
            ImGui::PopID();

            ImGui::Checkbox("Health Bar", &config.visuals.draw_health);
            ImGui::SameLine();
            ImGui::Checkbox("Armor Bar", &config.visuals.draw_armor);

            ImGui::Checkbox("Player Name", &config.visuals.draw_name);
            ImGui::SameLine();
            ImGui::Checkbox("Weapon Name", &config.visuals.draw_weapon);

            ImGui::Checkbox("Player Tags (helmet, defuser, bomb)", &config.visuals.draw_tags);

            ImGui::Checkbox("Dynamic Font Size", &config.visuals.dynamic_font);
            ImGui::DragFloat(
                "Static Font Size", &config.visuals.font_size, 0.02f, 1.0f, 50.0f, "%.1f");

            ImGui::Checkbox("Dropped Weapons", &config.visuals.dropped_weapons);
            ImGui::SameLine();
            ImGui::Checkbox("Sniper Crosshair", &config.visuals.sniper_crosshair);

            ImGui::DragFloat("Line Width", &config.visuals.line_width, 0.01f, 0.2f, 3.0f, "%.1f");

            ImGui::DragInt("Overlay FPS", &config.visuals.overlay_fps, 0.2f, 60, 240);

            ImGui::Checkbox("Debug Overlay", &config.visuals.debug_window);

            ImGui::EndChild();
            ImGui::EndTabItem();
        }

        if (ImGui::BeginTabItem("Unsafe")) {
            ImVec2 available = ImGui::GetContentRegionAvail();
            ImGui::BeginChild("tab_items_unsafe", available);

            ImGui::Checkbox("No Flash", &config.misc.no_flash);
            ImGui::DragFloat(
                "Max Flash Alpha", &config.misc.max_flash_alpha, 0.2f, 0.0f, 255.0f, "%.0f");

            ImGui::Checkbox("FOV Changer", &config.misc.fov_changer);
            ImGui::DragInt("Desired FOV", &config.misc.desired_fov, 0.2f, 1, 179);

            ImGui::EndChild();
            ImGui::EndTabItem();
        }

        if (ImGui::BeginTabItem("Colors")) {
            ImVec2 available = ImGui::GetContentRegionAvail();
            ImGui::BeginChild("tab_items_colors", available);

            ImGui::ColorEdit3("Text", &config.visuals.text_color.x);

            ImGui::ColorEdit3("Box", &config.visuals.box_color.x);

            ImGui::ColorEdit3("Skeleton", &config.visuals.skeleton_color.x);

            ImGui::ColorEdit3("Armor", &config.visuals.armor_color.x);

            ImGui::ColorEdit3("Crosshair", &config.visuals.crosshair_color.x);

            ImGui::EndChild();
            ImGui::EndTabItem();
        }

        if (ImGui::BeginTabItem("Config")) {
            ImVec2 available = ImGui::GetContentRegionAvail();
            ImGui::BeginChild("tab_items_config", available);

            if (ImGui::Button("Reset Config")) {
                ResetConfig();
            }

            ImGui::EndChild();
            ImGui::EndTabItem();
        }

        if (ImGui::BeginTabItem("Misc")) {
            ImVec2 available = ImGui::GetContentRegionAvail();
            ImGui::BeginChild("tab_items_misc", available);

            if (ImGui::Button("Report Issue")) {
                std::system("xdg-open https://github.com/avitran0/deadlocked");
            }

            ImGui::Text("Accent Color");

            PushButtonStyle(Colors::RED);
            if (ImGui::Button("Red")) {
                SetAccentColor(Colors::RED);
                config.accent_color = Colors::RED;
            }
            PopButtonStyle();

            ImGui::SameLine();

            PushButtonStyle(Colors::ORANGE);
            if (ImGui::Button("Orange")) {
                SetAccentColor(Colors::ORANGE);
                config.accent_color = Colors::ORANGE;
            }
            PopButtonStyle();

            ImGui::SameLine();

            PushButtonStyle(Colors::YELLOW);
            if (ImGui::Button("Yellow")) {
                SetAccentColor(Colors::YELLOW);
                config.accent_color = Colors::YELLOW;
            }
            PopButtonStyle();

            ImGui::SameLine();

            PushButtonStyle(Colors::GREEN);
            if (ImGui::Button("Green")) {
                SetAccentColor(Colors::GREEN);
                config.accent_color = Colors::GREEN;
            }
            PopButtonStyle();

            ImGui::SameLine();

            PushButtonStyle(Colors::CYAN);
            if (ImGui::Button("Cyan")) {
                SetAccentColor(Colors::CYAN);
                config.accent_color = Colors::CYAN;
            }
            PopButtonStyle();

            ImGui::SameLine();

            PushButtonStyle(Colors::BLUE);
            if (ImGui::Button("Blue")) {
                SetAccentColor(Colors::BLUE);
                config.accent_color = Colors::BLUE;
            }
            PopButtonStyle();

            ImGui::SameLine();

            PushButtonStyle(Colors::PURPLE);
            if (ImGui::Button("Purple")) {
                SetAccentColor(Colors::PURPLE);
                config.accent_color = Colors::PURPLE;
            }
            PopButtonStyle();

            ImGui::EndChild();
            ImGui::EndTabItem();
        }

        ImGui::EndTabBar();

        ImDrawList *gui_draw_list = ImGui::GetForegroundDrawList();
        std::string gui_fps = "FPS: " + std::to_string((i32)gui_io.Framerate);
        const ImVec2 text_size = ImGui::CalcTextSize(gui_fps.c_str());
        const ImVec2 gui_window_size = ImGui::GetWindowSize();
        gui_draw_list->AddText(
            ImVec2(gui_window_size.x - text_size.x - 4.0f, 12.0f), 0xFFFFFFFF, gui_fps.c_str());

        const ImVec2 version_text_size = ImGui::CalcTextSize(VERSION);
        gui_draw_list->AddText(
            ImVec2(
                gui_window_size.x - version_text_size.x - 4.0f,
                gui_window_size.y - version_text_size.y - 4.0f),
            0xFFFFFFFF, VERSION);

        ImGui::End();

        ImGui::Render();
        glViewport(0, 0, width, height);
        glClearColor(0.1176470592617989f, 0.1176470592617989f, 0.1568627506494522f, 1.0f);
        glClear(GL_COLOR_BUFFER_BIT);
        ImGui_ImplOpenGL3_RenderDrawData(ImGui::GetDrawData());

        SDL_GL_SwapWindow(gui_window);

        // overlay
        SDL_GL_MakeCurrent(overlay, overlay_gl);
        ImGui::SetCurrentContext(overlay_ctx);
        ImGui_ImplOpenGL3_NewFrame();
        ImGui_ImplSDL3_NewFrame();
        ImGui::NewFrame();

        ImGui::Begin(
            "overlay", nullptr,
            ImGuiWindowFlags_NoBackground | ImGuiWindowFlags_NoDecoration |
                ImGuiWindowFlags_NoInputs | ImGuiWindowFlags_NoResize | ImGuiWindowFlags_NoMove);
        ImGui::SetWindowPos(ImVec2(minX, minY));
        ImGui::SetWindowSize(ImVec2(maxX - minX, maxY - minY));
        ImDrawList *overlay_draw_list = ImGui::GetBackgroundDrawList();

        const ImU32 text_color = IM_COL32(
            config.visuals.text_color.x * 255, config.visuals.text_color.y * 255,
            config.visuals.text_color.z * 255, 255);

        std::string overlay_fps = "FPS: " + std::to_string((i32)overlay_io.Framerate);
        extern glm::ivec4 window_size;
        OutlineText(
            overlay_draw_list, ImVec2((f32)window_size.x + 4.0f, (f32)window_size.y + 4.0f),
            text_color, overlay_fps.c_str());

        if (config.visuals.debug_window) {
            // frame
            overlay_draw_list->AddRect(
                ImVec2(minX, minY), ImVec2(maxX - minX, maxY - minY), 0xFFFFFFFF, 0.0f, 0, 8.0f);

            // cross
            overlay_draw_list->AddLine(
                ImVec2(minX, minY), ImVec2(maxX - minX, maxY - minY), 0xFFFFFFFF, 4.0f);
            overlay_draw_list->AddLine(
                ImVec2(minX, maxY - minY), ImVec2(maxX - minX, minY), 0xFFFFFFFF, 4.0f);
        }
        if (config.visuals.enabled) {
            vinfo_lock.lock();
            for (auto player : player_info) {
                const ImU32 health_color = HealthColor(player.health);

                if (config.visuals.draw_skeleton != DrawStyle::DrawNone) {
                    ImU32 color;
                    if (config.visuals.draw_skeleton == DrawStyle::DrawColor) {
                        color = IM_COL32(
                            config.visuals.skeleton_color.x * 255,
                            config.visuals.skeleton_color.y * 255,
                            config.visuals.skeleton_color.z * 255, 255);
                    } else {
                        color = health_color;
                    }
                    for (auto connection : player.bones) {
                        const auto bone1 = WorldToScreen(connection.first);
                        const auto bone2 = WorldToScreen(connection.second);
                        if (bone1.has_value() && bone2.has_value()) {
                            const ImVec2 start = ImVec2(bone1.value().x, bone1.value().y);
                            const ImVec2 end = ImVec2(bone2.value().x, bone2.value().y);
                            overlay_draw_list->AddLine(
                                start, end, color, config.visuals.line_width);
                        }
                    }
                }

                const auto bottom_opt = WorldToScreen(player.position);
                const auto top_opt = WorldToScreen(player.head + glm::vec3(0.0f, 0.0f, 8.0f));

                if (!bottom_opt.has_value() || !top_opt.has_value()) {
                    continue;
                }

                const ImVec2 bottom = ImVec2(bottom_opt.value().x, bottom_opt.value().y);
                const ImVec2 top = ImVec2(bottom.x, bottom.y + (top_opt.value().y - bottom.y));

                const f32 height = bottom.y - top.y;
                const f32 width = height / 2.0f;
                const f32 half_width = width / 2.0f;
                f32 font_size =
                    (config.visuals.dynamic_font ? half_width : config.visuals.font_size) * scale;
                if (font_size > 20.0f * scale) {
                    font_size = 20.0f * scale;
                }
                ImFont *font = overlay_io.Fonts->Fonts[0];

                const ImVec2 bottom_left = ImVec2(bottom.x - half_width, bottom.y);
                const ImVec2 bottom_right = ImVec2(bottom.x + half_width, bottom.y);
                const ImVec2 top_left = ImVec2(top.x - half_width, top.y);
                const ImVec2 top_right = ImVec2(top.x + half_width, top.y);

                if (config.visuals.draw_box != DrawStyle::DrawNone) {
                    // four corners, each a quarter of the width/height
                    // convert imvec4 to imu32
                    ImU32 color;
                    if (config.visuals.draw_box == DrawStyle::DrawColor) {
                        color = IM_COL32(
                            config.visuals.box_color.x * 255, config.visuals.box_color.y * 255,
                            config.visuals.box_color.z * 255, 255);
                    } else {
                        color = health_color;
                    }
                    overlay_draw_list->AddLine(
                        bottom_left, ImVec2(bottom_left.x, bottom_left.y - height / 4.0f), color,
                        config.visuals.line_width);
                    overlay_draw_list->AddLine(
                        bottom_left, ImVec2(bottom_left.x + width / 4.0f, bottom_left.y), color,
                        config.visuals.line_width);
                    overlay_draw_list->AddLine(
                        bottom_right, ImVec2(bottom_right.x, bottom_right.y - height / 4.0f), color,
                        config.visuals.line_width);
                    overlay_draw_list->AddLine(
                        bottom_right, ImVec2(bottom_right.x - width / 4.0f, bottom_right.y), color,
                        config.visuals.line_width);
                    overlay_draw_list->AddLine(
                        top_left, ImVec2(top_left.x, top_left.y + height / 4.0f), color,
                        config.visuals.line_width);
                    overlay_draw_list->AddLine(
                        top_left, ImVec2(top_left.x + width / 4.0f, top_left.y), color,
                        config.visuals.line_width);
                    overlay_draw_list->AddLine(
                        top_right, ImVec2(top_right.x, top_right.y + height / 4.0f), color,
                        config.visuals.line_width);
                    overlay_draw_list->AddLine(
                        top_right, ImVec2(top_right.x - width / 4.0f, top_right.y), color,
                        config.visuals.line_width);
                }

                if (config.visuals.draw_health) {
                    const ImVec2 health_bottom = ImVec2(bottom_left.x - 4.0f, bottom_left.y);
                    // adjust height based on health
                    const ImVec2 health_top = ImVec2(
                        top_left.x - 4.0f, bottom_left.y - height * (f32)player.health / 100.0f);
                    overlay_draw_list->AddLine(
                        health_bottom, health_top, health_color, config.visuals.line_width);
                }

                if (config.visuals.draw_armor) {
                    const ImVec2 armor_bottom = ImVec2(bottom_left.x - 8, bottom_left.y);
                    const ImVec2 armor_top =
                        ImVec2(top_left.x - 8, bottom_left.y - height * (f32)player.armor / 100.0f);
                    overlay_draw_list->AddLine(
                        armor_bottom, armor_top,
                        IM_COL32(
                            config.visuals.armor_color.x * 255, config.visuals.armor_color.y * 255,
                            config.visuals.armor_color.z * 255, 255),
                        config.visuals.line_width);
                }

                if (config.visuals.draw_name) {
                    const ImVec2 text_size =
                        font->CalcTextSizeA(font_size, FLT_MAX, 0.0f, player.name.c_str());
                    const ImVec2 name_position =
                        ImVec2(top.x - text_size.x / 2.0f, top_left.y - font_size);
                    OutlineText(
                        overlay_draw_list, font, font_size, name_position, text_color,
                        player.name.c_str());
                }

                if (config.visuals.draw_weapon) {
                    const ImVec2 text_size =
                        font->CalcTextSizeA(font_size, FLT_MAX, 0.0f, player.weapon.c_str());
                    const ImVec2 weapon_position =
                        ImVec2(bottom.x - text_size.x / 2.0f, bottom_left.y);
                    OutlineText(
                        overlay_draw_list, font, font_size, weapon_position, text_color,
                        player.weapon.c_str());
                }

                f32 offset = font_size;

                if (config.visuals.draw_tags && player.has_helmet) {
                    const ImVec2 text_size =
                        font->CalcTextSizeA(font_size, FLT_MAX, 0.0f, "helmet");
                    const ImVec2 helmet_position =
                        ImVec2(bottom.x - text_size.x / 2.0f, bottom_left.y + offset);
                    offset += font_size;
                    OutlineText(
                        overlay_draw_list, font, font_size, helmet_position, text_color, "helmet");
                }

                if (config.visuals.draw_tags && player.has_defuser) {
                    const ImVec2 text_size =
                        font->CalcTextSizeA(font_size, FLT_MAX, 0.0f, "defuser");
                    const ImVec2 defuser_position =
                        ImVec2(bottom.x - text_size.x / 2.0f, bottom_left.y + offset);
                    offset += font_size;
                    OutlineText(
                        overlay_draw_list, font, font_size, defuser_position, text_color,
                        "defuser");
                }

                if (config.visuals.draw_tags && player.has_bomb) {
                    const ImVec2 text_size = font->CalcTextSizeA(font_size, FLT_MAX, 0.0f, "bomb");
                    const ImVec2 bomb_position =
                        ImVec2(bottom.x - text_size.x / 2.0f, bottom_left.y + offset);
                    offset += font_size;
                    OutlineText(
                        overlay_draw_list, font, font_size, bomb_position, text_color, "bomb");
                }
            }

            if (config.visuals.dropped_weapons) {
                for (auto entity : entity_info) {
                    const auto position_opt = WorldToScreen(entity.position);
                    if (!position_opt.has_value()) {
                        continue;
                    }
                    const auto position = position_opt.value();
                    OutlineText(
                        overlay_draw_list, ImVec2(position.x, position.y), 0xFFFFFFFF,
                        entity.name.c_str());
                }
            }

            extern MiscInfo misc_info;
            // fov circle
            if (config.aimbot.fov_circle && misc_info.in_game) {
                const f32 pawn_fov = config.misc.fov_changer ? config.misc.desired_fov : 90.0f;
                const f32 radius = tan(config.aimbot.fov / 180.f * M_PI / 2.f) /
                                   tan(pawn_fov / 180.f * M_PI / 2.f) * window_size.x;
                const ImVec2 center = ImVec2(
                    window_size.x + window_size.z / 2.0f, window_size.y + window_size.w / 2.0f);
                overlay_draw_list->AddCircle(center, radius, 0xFFFFFFFF);
            }

            // sniper crosshair
            if (config.visuals.sniper_crosshair &&
                WeaponClassFromString(misc_info.held_weapon) == WeaponClass::Sniper) {
                const f32 crosshair_size = 32.0f;
                const ImVec2 center = ImVec2(
                    (f32)window_size.x + (f32)window_size.z / 2.0f,
                    (f32)window_size.y + (f32)window_size.w / 2.0f);
                const ImU32 color = IM_COL32(
                    config.visuals.crosshair_color.x * 255, config.visuals.crosshair_color.y * 255,
                    config.visuals.crosshair_color.z * 255, 255);
                overlay_draw_list->AddLine(
                    ImVec2(center.x - crosshair_size, center.y),
                    ImVec2(center.x + crosshair_size, center.y), color);
                overlay_draw_list->AddLine(
                    ImVec2(center.x, center.y - crosshair_size),
                    ImVec2(center.x, center.y + crosshair_size), color);
            }

            vinfo_lock.unlock();
        }

        ImGui::End();
        config_lock.unlock();

        ImGui::Render();
        SDL_GetWindowSize(overlay, &width, &height);
        glViewport(0, 0, width, height);
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
        const auto frame_time = std::chrono::microseconds(1000000 / config.visuals.overlay_fps);
        std::this_thread::sleep_for(frame_time - us);
        // glfwPollEvents();
    }

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

    // todo: why are these pointers invalid?
    // ImGui::DestroyContext(gui_ctx);
    // ImGui::DestroyContext(overlay_ctx);

    SDL_GL_DestroyContext(gui_gl);
    SDL_GL_DestroyContext(overlay_gl);
    SDL_DestroyWindow(gui_window);
    SDL_DestroyWindow(overlay);
    SDL_Quit();
}
