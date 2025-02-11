#include "gui.hpp"

#include <SDL3/SDL.h>
#include <SDL3/SDL_opengl.h>
#include <imgui.h>
#include <imgui_impl_opengl3.h>
#include <imgui_impl_sdl3.h>

#include <chrono>
#include <filesystem>
#include <log.hpp>
#include <thread>

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

void Gui() {
    SDL_SetHint(SDL_HINT_VIDEO_DRIVER, "x11");

    if (!SDL_Init(SDL_INIT_VIDEO)) {
        Log(LogLevel::Error, "sdl3 initialization failed, exiting");
        exit(1);
    }

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

    Log(LogLevel::Info, "screen top left corner at: " + std::to_string(minX) + " x " + std::to_string(minY) + " px");
    Log(LogLevel::Info,
        "screen resolution: " + std::to_string(maxX - minX) + " x " + std::to_string(maxY - minY) + " px");

    IMGUI_CHECKVERSION();
    ImGuiContext *gui_ctx = ImGui::CreateContext();
    ImGuiContext *overlay_ctx = ImGui::CreateContext();

    SDL_GL_SetAttribute(SDL_GL_CONTEXT_PROFILE_MASK, SDL_GL_CONTEXT_PROFILE_CORE);

    // gui window
    SDL_Window *gui_window = SDL_CreateWindow("deadlocked", 640, 420, SDL_WINDOW_OPENGL);
    if (!gui_window) {
        Log(LogLevel::Error, "could not create gui window");
        return;
    }
    SDL_SetWindowPosition(gui_window, SDL_WINDOWPOS_CENTERED, SDL_WINDOWPOS_CENTERED);
    SDL_GLContext gui_gl = SDL_GL_CreateContext(gui_window);
    SDL_GL_MakeCurrent(gui_window, gui_gl);
    SDL_GL_SetSwapInterval(0);

    SDL_Window *temp = SDL_CreateWindow("deadlocked", 1, 1, SDL_WINDOW_BORDERLESS);
    if (!temp) {
        Log(LogLevel::Error, "could not create overlay window");
        return;
    }
    SDL_SetWindowPosition(temp, 0, 0);

    // overlay window
    SDL_Window *overlay =
        SDL_CreatePopupWindow(temp, 0, 0, maxX - minX, maxY - minY,
                              SDL_WINDOW_ALWAYS_ON_TOP | SDL_WINDOW_BORDERLESS | SDL_WINDOW_NOT_FOCUSABLE |
                                  SDL_WINDOW_OPENGL | SDL_WINDOW_TOOLTIP | SDL_WINDOW_TRANSPARENT);
    if (!overlay) {
        Log(LogLevel::Error, "could not create overlay window");
        return;
    }
    SDL_SetWindowPosition(overlay, minX, minY);
    SDL_GLContext overlay_gl = SDL_GL_CreateContext(overlay);
    SDL_GL_MakeCurrent(overlay, overlay_gl);
    SDL_GL_SetSwapInterval(0);

    SDL_SetWindowPosition(temp, 0, 0);

    SDL_ShowWindow(overlay);
    SDL_ShowWindow(gui_window);

    ImGui::SetCurrentContext(gui_ctx);
    Style();

    ImGuiIO &gui_io = ImGui::GetIO();
    gui_io.IniFilename = nullptr;
    if (std::filesystem::exists(std::filesystem::path(FONT))) {
        gui_io.Fonts->AddFontFromMemoryTTF(jet_brains_mono, jet_brains_mono_len, 20);
    } else {
        Log(LogLevel::Warning, "font not found: " + std::string(FONT));
    }

    ImGui_ImplSDL3_InitForOpenGL(gui_window, gui_gl);
    ImGui_ImplOpenGL3_Init("#version 130");

    ImGui::SetCurrentContext(overlay_ctx);
    Style();

    ImGuiIO &overlay_io = ImGui::GetIO();
    overlay_io.IniFilename = nullptr;
    if (std::filesystem::exists(std::filesystem::path(FONT))) {
        overlay_io.Fonts->AddFontFromMemoryTTF(jet_brains_mono, jet_brains_mono_len, 20);
    } else {
        Log(LogLevel::Warning, "font not found: " + std::string(FONT));
    }

    ImGui_ImplSDL3_InitForOpenGL(overlay, overlay_gl);
    ImGui_ImplOpenGL3_Init("#version 130");

    std::thread cs2(CS2);

    bool should_close = false;
    while (!should_close) {
        auto clock = std::chrono::steady_clock::now();

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
        ImGui::Begin("deadlocked", nullptr,
                     ImGuiWindowFlags_NoResize | ImGuiWindowFlags_NoDecoration | ImGuiWindowFlags_NoMove);
        ImGui::SetWindowSize(ImVec2(width, height));
        ImGui::SetWindowPos(ImVec2(0.0f, 0.0f));

        // tabs
        config_lock.lock();
        ImGui::BeginTabBar("tabs");

        if (ImGui::BeginTabItem("Aimbot")) {
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

            ImGui::Checkbox("Visibility Check", &config.aimbot.visibility_check);
            ImGui::DragFloat("FOV", &config.aimbot.fov, 0.02f, 0.1f, 360.0f, "%.1f");

            ImGui::Checkbox("Aim Lock", &config.aimbot.aim_lock);
            ImGui::DragFloat("Smooth", &config.aimbot.smooth, 0.02f, 1.0f, 10.0f, "%.1f");

            ImGui::Checkbox("RCS", &config.aimbot.rcs);

            ImGui::EndTabItem();
        }

        if (ImGui::BeginTabItem("Visuals")) {
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
            if (ImGui::RadioButton("Health", config.visuals.draw_skeleton == DrawStyle::DrawHealth)) {
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

            ImGui::Checkbox("Dropped Weapons", &config.visuals.dropped_weapons);

            ImGui::DragFloat("Line Width", &config.visuals.line_width, 0.01f, 0.2f, 3.0f, "%.1f");

            ImGui::DragInt("Overlay FPS", &config.visuals.overlay_fps, 0.2f, 60, 240);

            ImGui::Checkbox("Debug Overlay", &config.visuals.debug_window);

            ImGui::EndTabItem();
        }

        if (ImGui::BeginTabItem("Unsafe")) {
            ImGui::Checkbox("No Flash", &config.misc.no_flash);
            ImGui::DragFloat("Max Flash Alpha", &config.misc.max_flash_alpha, 0.2f, 0.0f, 255.0f, "%.0f");

            ImGui::Checkbox("FOV Changer", &config.misc.fov_changer);
            ImGui::DragInt("Desired FOV", &config.misc.desired_fov, 0.2f, 1, 179);

            ImGui::EndTabItem();
        }

        if (ImGui::BeginTabItem("Colors")) {
            ImGui::ColorEdit3("Box", &config.visuals.box_color.x);

            ImGui::ColorEdit3("Skeleton", &config.visuals.skeleton_color.x);

            ImGui::ColorEdit3("Armor", &config.visuals.armor_color.x);

            ImGui::EndTabItem();
        }

        ImGui::EndTabBar();

        if (ImGui::Button("reset config")) {
            ResetConfig();
        }

        ImDrawList *gui_draw_list = ImGui::GetForegroundDrawList();
        ImGui::SetCursorScreenPos(ImVec2(0.0f, 0.0f));
        std::string gui_fps = "FPS: " + std::to_string((i32)gui_io.Framerate);
        const ImVec2 text_size = ImGui::CalcTextSize(gui_fps.c_str());
        const ImVec2 gui_window_size = ImGui::GetWindowSize();
        gui_draw_list->AddText(ImVec2(gui_window_size.x - text_size.x - 4.0f, gui_window_size.y - text_size.y - 4.0f),
                               0xFFFFFFFF, gui_fps.c_str());

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

        ImGui::Begin("overlay", nullptr,
                     ImGuiWindowFlags_NoBackground | ImGuiWindowFlags_NoDecoration | ImGuiWindowFlags_NoInputs |
                         ImGuiWindowFlags_NoResize | ImGuiWindowFlags_NoMove);
        ImGui::SetWindowPos(ImVec2(minX, minY));
        ImGui::SetWindowSize(ImVec2(maxX - minX, maxY - minY));
        ImDrawList *overlay_draw_list = ImGui::GetBackgroundDrawList();

        std::string overlay_fps = "FPS: " + std::to_string((i32)overlay_io.Framerate);
        extern glm::ivec4 window_size;
        overlay_draw_list->AddText(ImVec2((f32)window_size.x + 4.0f, (f32)window_size.y + 4.0f), 0xFFFFFFFF,
                                   overlay_fps.c_str());

        if (config.visuals.debug_window) {
            // frame
            overlay_draw_list->AddRect(ImVec2(minX, minY), ImVec2(maxX - minX, maxY - minY), 0xFFFFFFFF, 0.0f, 0, 8.0f);

            // cross
            overlay_draw_list->AddLine(ImVec2(minX, minY), ImVec2(maxX - minX, maxY - minY), 0xFFFFFFFF, 4.0f);
            overlay_draw_list->AddLine(ImVec2(minX, maxY - minY), ImVec2(maxX - minX, minY), 0xFFFFFFFF, 4.0f);
        }

        vinfo_lock.lock();
        for (auto player : player_info) {
            const ImU32 health_color = HealthColor(player.health);

            if (config.visuals.draw_skeleton != DrawStyle::DrawNone) {
                ImU32 color;
                if (config.visuals.draw_skeleton == DrawStyle::DrawColor) {
                    color = IM_COL32(config.visuals.skeleton_color.x * 255, config.visuals.skeleton_color.y * 255,
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
                        overlay_draw_list->AddLine(start, end, color, config.visuals.line_width);
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

            const ImVec2 bottom_left = ImVec2(bottom.x - half_width, bottom.y);
            const ImVec2 bottom_right = ImVec2(bottom.x + half_width, bottom.y);
            const ImVec2 top_left = ImVec2(top.x - half_width, top.y);
            const ImVec2 top_right = ImVec2(top.x + half_width, top.y);

            if (config.visuals.draw_box != DrawStyle::DrawNone) {
                // four corners, each a quarter of the width/height
                // convert imvec4 to imu32
                ImU32 color;
                if (config.visuals.draw_box == DrawStyle::DrawColor) {
                    color = IM_COL32(config.visuals.box_color.x * 255, config.visuals.box_color.y * 255,
                                     config.visuals.box_color.z * 255, 255);
                } else {
                    color = health_color;
                }
                overlay_draw_list->AddLine(bottom_left, ImVec2(bottom_left.x, bottom_left.y - height / 4.0f), color,
                                           config.visuals.line_width);
                overlay_draw_list->AddLine(bottom_left, ImVec2(bottom_left.x + width / 4.0f, bottom_left.y), color,
                                           config.visuals.line_width);
                overlay_draw_list->AddLine(bottom_right, ImVec2(bottom_right.x, bottom_right.y - height / 4.0f), color,
                                           config.visuals.line_width);
                overlay_draw_list->AddLine(bottom_right, ImVec2(bottom_right.x - width / 4.0f, bottom_right.y), color,
                                           config.visuals.line_width);
                overlay_draw_list->AddLine(top_left, ImVec2(top_left.x, top_left.y + height / 4.0f), color,
                                           config.visuals.line_width);
                overlay_draw_list->AddLine(top_left, ImVec2(top_left.x + width / 4.0f, top_left.y), color,
                                           config.visuals.line_width);
                overlay_draw_list->AddLine(top_right, ImVec2(top_right.x, top_right.y + height / 4.0f), color,
                                           config.visuals.line_width);
                overlay_draw_list->AddLine(top_right, ImVec2(top_right.x - width / 4.0f, top_right.y), color,
                                           config.visuals.line_width);
            }

            if (config.visuals.draw_health) {
                const ImVec2 health_bottom = ImVec2(bottom_left.x - 4.0f, bottom_left.y);
                // adjust height based on health
                const ImVec2 health_top =
                    ImVec2(top_left.x - 4.0f, bottom_left.y - height * (f32)player.health / 100.0f);
                overlay_draw_list->AddLine(health_bottom, health_top, health_color, config.visuals.line_width);
            }

            if (config.visuals.draw_armor) {
                const ImVec2 armor_bottom = ImVec2(bottom_left.x - 8, bottom_left.y);
                const ImVec2 armor_top = ImVec2(top_left.x - 8, bottom_left.y - height * (f32)player.armor / 100.0f);
                overlay_draw_list->AddLine(
                    armor_bottom, armor_top,
                    IM_COL32(config.visuals.armor_color.x * 255, config.visuals.armor_color.y * 255,
                             config.visuals.armor_color.z * 255, 255),
                    config.visuals.line_width);
            }

            if (config.visuals.draw_name) {
                const ImVec2 name_position = ImVec2(top_left.x, top_left.y - 18.0f);
                overlay_draw_list->AddText(name_position, 0xFFFFFFFF, player.name.c_str());
            }

            if (config.visuals.draw_weapon) {
                const ImVec2 weapon_position = ImVec2(bottom_left.x, bottom_left.y);
                overlay_draw_list->AddText(weapon_position, 0xFFFFFFFF, player.weapon.c_str());
            }

            f32 offset = 16.0f;

            if (config.visuals.draw_tags && player.has_helmet) {
                const ImVec2 helmet_position = ImVec2(bottom_left.x, bottom_left.y + offset);
                offset += 16.0f;
                overlay_draw_list->AddText(helmet_position, 0xFFFFFFFF, "helmet");
            }

            if (config.visuals.draw_tags && player.has_defuser) {
                const ImVec2 defuser_position = ImVec2(bottom_left.x, bottom_left.y + offset);
                offset += 16.0f;
                overlay_draw_list->AddText(defuser_position, 0xFFFFFFFF, "defuser");
            }

            if (config.visuals.draw_tags && player.has_bomb) {
                const ImVec2 bomb_position = ImVec2(bottom_left.x, bottom_left.y + offset);
                offset += 16.0f;
                overlay_draw_list->AddText(bomb_position, 0xFFFFFFFF, "bomb");
            }
        }

        if (config.visuals.dropped_weapons) {
            for (auto entity : entity_info) {
                const auto position_opt = WorldToScreen(entity.position);
                if (!position_opt.has_value()) {
                    continue;
                }
                const auto position = position_opt.value();
                overlay_draw_list->AddText(ImVec2(position.x, position.y), 0xFFFFFFFF, entity.name.c_str());
            }
        }
        vinfo_lock.unlock();

        ImGui::End();
        config_lock.unlock();

        ImGui::Render();
        SDL_GetWindowSize(overlay, &width, &height);
        glViewport(0, 0, width, height);
        glClearColor(0.0f, 0.0f, 0.0f, 0.0f);
        glClear(GL_COLOR_BUFFER_BIT);
        ImGui_ImplOpenGL3_RenderDrawData(ImGui::GetDrawData());

        SDL_GL_SwapWindow(overlay);

        SaveConfig();

        const auto end_time = std::chrono::steady_clock::now();
        const auto us = std::chrono::duration_cast<std::chrono::microseconds>(end_time - clock);
        const auto frame_time = std::chrono::microseconds(1000000 / config.visuals.overlay_fps);
        std::this_thread::sleep_for(frame_time - us);
        // glfwPollEvents();
    }

    config_lock.lock();
    extern bool should_quit;
    should_quit = true;
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
