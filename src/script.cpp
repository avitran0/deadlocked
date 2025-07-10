#include "script.hpp"

extern "C" {
#include <lualib.h>
}

#include <filesystem>
#include <fstream>
#include <sstream>
#include <utility>
#include <vector>

#include "key_code.hpp"
#include "logging.hpp"
#include "types.hpp"

Script script;

i32 RegisterOnce(lua_State *state) {
    luaL_checktype(state, 1, LUA_TFUNCTION);
    lua_pushvalue(state, 1);
    const i32 ref = luaL_ref(state, LUA_REGISTRYINDEX);
    script.exec_once.push_back(ref);
    return 0;
}

i32 RegisterTick(lua_State *state) {
    luaL_checktype(state, 1, LUA_TFUNCTION);
    lua_pushvalue(state, 1);
    const i32 ref = luaL_ref(state, LUA_REGISTRYINDEX);
    script.exec_tick.push_back(ref);
    return 0;
}

i32 RegisterKeyHeld(lua_State *state) {
    const i32 key = luaL_checkinteger(state, 1);
    luaL_checktype(state, 2, LUA_TFUNCTION);
    lua_pushvalue(state, 2);
    const i32 ref = luaL_ref(state, LUA_REGISTRYINDEX);
    script.exec_key_held.emplace_back(static_cast<KeyCode>(key), ref);
    return 0;
}

i32 RegisterKeyPressed(lua_State *state) {
    const i32 key = luaL_checkinteger(state, 1);
    luaL_checktype(state, 2, LUA_TFUNCTION);
    lua_pushvalue(state, 2);
    const i32 ref = luaL_ref(state, LUA_REGISTRYINDEX);
    script.exec_key_pressed.emplace_back(static_cast<KeyCode>(key), ref);
    return 0;
}

using LuaFunc = i32 (*)(lua_State *state);
std::vector<std::pair<const char *, LuaFunc>> functions = {
    {"register_once", RegisterOnce},
    {"register_tick", RegisterTick},
    {"register_key_held", RegisterKeyHeld},
    {"register_key_pressed", RegisterKeyPressed}};

Script::Script() {
    state = luaL_newstate();
    if (!state) {
        logging::Error("could not initialize scripting engine");
        std::exit(1);
    }
    luaL_openlibs(state);
    for (const auto &[name, func] : functions) {
        lua_register(state, name, func);
    }

    const auto exe = std::filesystem::canonical("/proc/self/exe");
    const auto path = exe.parent_path() / "scripts";
    if (!std::filesystem::exists(path)) {
        std::filesystem::create_directory(path);
    }

    for (const auto &entry : std::filesystem::directory_iterator(path)) {
        if (!entry.is_regular_file()) {
            continue;
        }
        if (entry.path().filename().extension() == ".lua") {
            std::ostringstream input;
            std::ifstream file(path);
            input << file.rdbuf();
            scripts.push_back(input.str());
        }
    }
}

void Script::RunFunction(const i32 ref) {
    lua_rawgeti(state, LUA_REGISTRYINDEX, ref);
    if (lua_pcall(state, 0, 0, 0) != LUA_OK) {
        logging::Warning("error in lua function: {}", lua_tostring(state, -1));
        lua_pop(state, 1);
    }
}
