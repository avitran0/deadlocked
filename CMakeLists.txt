cmake_minimum_required(VERSION 3.28)
project(deadlocked CXX)

set(CMAKE_CXX_STANDARD 17)

if(NOT CMAKE_SYSTEM_NAME STREQUAL "Linux")
    message(FATAL_ERROR "this only works on linux.")
endif()

file(GLOB_RECURSE SOURCE_FILES "src/*.cpp")

add_executable(deadlocked ${SOURCE_FILES})
target_include_directories(deadlocked PRIVATE include)

target_include_directories(deadlocked PRIVATE lib/imgui)
target_include_directories(deadlocked PRIVATE lib/imgui/backends)
set(IMGUI_SOURCE_FILES lib/imgui/imgui.cpp lib/imgui/imgui_demo.cpp lib/imgui/imgui_draw.cpp
    lib/imgui/imgui_tables.cpp lib/imgui/imgui_widgets.cpp
    lib/imgui/backends/imgui_impl_sdl3.cpp lib/imgui/backends/imgui_impl_opengl3.cpp)
set_source_files_properties(${IMGUI_SOURCE_FILES} PROPERTIES COMPILE_FLAGS $<$<CONFIG:Debug>:-g0>)
target_sources(deadlocked PRIVATE ${IMGUI_SOURCE_FILES})

set(SDL_TESTS OFF CACHE BOOL "disable sdl tests" FORCE)
set(SDL_EXAMPLES OFF CACHE BOOL "disable sdl examples" FORCE)
set(SDL_INSTALL_DOCS OFF CACHE BOOL "disable sdl docs" FORCE)

# required for curl?
find_package(ICU REQUIRED COMPONENTS uc i18n data)

add_subdirectory(lib/glm)
add_subdirectory(lib/sdl)
add_subdirectory(lib/mithril)

target_link_libraries(deadlocked PRIVATE ICU::uc ICU::i18n ICU::data)
target_link_libraries(deadlocked PRIVATE glm::glm)
target_link_libraries(deadlocked PRIVATE OpenGL)
target_link_libraries(deadlocked PRIVATE SDL3::SDL3)
target_link_libraries(deadlocked PRIVATE mithril)

# valgrind/callgrind does not work with native, but with skylake it does
target_compile_options(deadlocked PRIVATE "-march=native")

set(CMAKE_EXPORT_COMPILE_COMMANDS 1)
