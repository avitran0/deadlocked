#include "mouse.hpp"

#include <fcntl.h>
#include <linux/input-event-codes.h>
#include <linux/input.h>
#include <unistd.h>

#include <filesystem>
#include <fstream>
#include <iostream>

#include "log.hpp"
#include "types.hpp"

i32 mouse = 0;

void MouseInit() {
    for (const auto &entry : std::filesystem::directory_iterator("/dev/input")) {
        if (!entry.is_character_file()) {
            continue;
        }

        const std::string event_name = entry.path().filename().string();
        if (event_name.rfind("event", 0) != 0) {
            continue;
        }

        const std::string path = "/sys/class/input/" + event_name + "/device/capabilities/rel";
        std::ifstream rel_file(path);
        if (!rel_file.is_open()) {
            continue;
        }

        std::string hex_str;
        rel_file >> hex_str;
        rel_file.close();

        u64 caps = 0;
        std::stringstream ss;
        ss << std::hex << hex_str;
        ss >> caps;

        // Check whether the REL_X (bit 0) and REL_Y (bit 1) bits are set.
        bool has_rel_x = (caps & (1 << REL_X)) != 0;
        bool has_rel_y = (caps & (1 << REL_Y)) != 0;

        if (!has_rel_x || !has_rel_y) {
            continue;
        }

        const std::string name_path = "/sys/class/input/" + event_name + "/device/name";
        std::ifstream name_file(name_path);
        if (!name_file.is_open()) {
            continue;
        }

        std::string device_name;
        std::getline(name_file, device_name);
        name_file.close();

        mouse = open(entry.path().c_str(), O_WRONLY);

        Log(LogLevel::Info, "found mouse: " + device_name);
        return;
    }

    mouse = open("/dev/null", O_WRONLY);
    Log(LogLevel::Warning, "no mouse was found");
}

void MouseMove(const glm::ivec2 &coords) {
    Log(LogLevel::Debug,
        "mouse move: " + std::to_string(coords.x) + "/" + std::to_string(coords.y));
    struct input_event ev {};

    // x
    ev.type = EV_REL;
    ev.code = REL_X;
    ev.value = coords.x;
    write(mouse, &ev, sizeof(ev));

    // y
    ev.type = EV_REL;
    ev.code = REL_Y;
    ev.value = coords.y;
    write(mouse, &ev, sizeof(ev));

    // syn
    ev.type = EV_SYN;
    ev.code = SYN_REPORT;
    ev.value = 0;
    write(mouse, &ev, sizeof(ev));
}

void MouseLeftPress() {
    Log(LogLevel::Debug, "pressed left mouse button");
    struct input_event ev {};

    // y
    ev.type = EV_KEY;
    ev.code = BTN_LEFT;
    ev.value = 1;
    write(mouse, &ev, sizeof(ev));

    // syn
    ev.type = EV_SYN;
    ev.code = SYN_REPORT;
    ev.value = 0;
    write(mouse, &ev, sizeof(ev));
}

void MouseLeftRelease() {
    Log(LogLevel::Debug, "released left mouse button");
    struct input_event ev {};

    // y
    ev.type = EV_KEY;
    ev.code = BTN_LEFT;
    ev.value = 0;
    write(mouse, &ev, sizeof(ev));

    // syn
    ev.type = EV_SYN;
    ev.code = SYN_REPORT;
    ev.value = 0;
    write(mouse, &ev, sizeof(ev));
}
