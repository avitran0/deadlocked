cmake_minimum_required(VERSION 3.30)
project(deadlocked LANGUAGES C CXX)

set(CMAKE_CXX_STANDARD 17)

if(NOT CMAKE_SYSTEM_NAME STREQUAL "Linux")
    message(FATAL_ERROR "this only works on linux.")
endif()

file(GLOB_RECURSE SOURCE_FILES "src/*.cpp")

add_executable(deadlocked ${SOURCE_FILES})
target_include_directories(deadlocked PRIVATE include)

set(BUILD_SHARED_LIBS OFF)
set(SDL_STATIC ON)
set(SDL_SHARED OFF)
set(SDL_TESTS OFF)
set(SDL_EXAMPLES OFF)
set(SDL_INSTALL_DOCS OFF)

add_subdirectory(lib)

target_link_libraries(deadlocked PRIVATE imgui glm::glm
                        OpenGL SDL3::SDL3 mithril lua)

# valgrind/callgrind does not work with native, but with skylake it does
target_compile_options(deadlocked PRIVATE "-march=native")

set(CMAKE_EXPORT_COMPILE_COMMANDS 1)
