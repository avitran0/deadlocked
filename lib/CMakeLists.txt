cmake_minimum_required(VERSION 3.30)

set(IMGUI_SOURCES imgui/imgui.cpp imgui/imgui_demo.cpp imgui/imgui_draw.cpp
    imgui/imgui_tables.cpp imgui/imgui_widgets.cpp
    imgui/backends/imgui_impl_sdl3.cpp imgui/backends/imgui_impl_opengl3.cpp)
add_library(imgui STATIC ${IMGUI_SOURCES})
target_include_directories(imgui PUBLIC "imgui")
target_include_directories(imgui PUBLIC "imgui/backends")

add_subdirectory(glm)
add_subdirectory(sdl)
add_subdirectory(mithril)

target_link_libraries(imgui PUBLIC SDL3::SDL3)

file(GLOB_RECURSE LUA_SOURCES "lua/*.c")
list(REMOVE_ITEM LUA_SOURCES "${CMAKE_CURRENT_SOURCE_DIR}/lua/onelua.c")
add_library(lua STATIC ${LUA_SOURCES})
target_include_directories(lua PUBLIC "lua")
