#include "mouse.hpp"

#include <fcntl.h>
#include <linux/input-event-codes.h>
#include <linux/input.h>
#include <sys/ioctl.h>
#include <unistd.h>

#include <cerrno>
#include <filesystem>
#include <fstream>
#include <mithril/logging.hpp>
#include <mithril/types.hpp>

#include "process.hpp"
#include "stealthmem.hpp"

i32 mouse = -1;
bool kernel = KernelModuleActive();
std::vector<std::pair<std::string, std::string>> input_devices;
std::pair<std::string, std::string> active_device;

std::vector<bool> HexToReversedBinary(const char hex_char) {
    int value = 0;
    if (hex_char >= '0' && hex_char <= '9') {
        value = hex_char - '0';
    } else if (hex_char >= 'a' && hex_char <= 'f') {
        value = hex_char - 'a' + 10;
    } else if (hex_char >= 'A' && hex_char <= 'F') {
        value = hex_char - 'A' + 10;
    }

    std::vector<bool> reversed_bits(4);
    for (int i = 0; i < 4; i++) {
        reversed_bits[i] = (value >> i & 1) != 0;
    }
    return reversed_bits;
}

std::vector<bool> DecodeCapabilities(const std::string &filename) {
    std::ifstream in(filename);
    if (!in.good()) {
        logging::Warning("cannot open mouse capability device file: {}", filename);
        return {};
    }

    std::string content;
    std::getline(in, content);

    std::vector<bool> binary_out;
    // current hex digits in group (max 16)
    int hex_count = 0;

    // line has to be processed right to left (why tf?)
    for (auto it = content.rbegin(); it != content.rend(); ++it) {
        char c = *it;
        if (c == '\n') {
            continue;
        }
        if (c == ' ') {
            // end group on spaces, and pad accordingly
            int padding_bits = 4 * (16 - hex_count);
            for (int i = 0; i < padding_bits; i++) {
                binary_out.push_back(false);
            }
            hex_count = 0;
        } else if (std::isxdigit(c)) {
            std::vector<bool> bits = HexToReversedBinary(c);
            for (bool bit : bits) {
                binary_out.push_back(bit);
            }
            hex_count++;
        }
    }

    // last group might have to be padded as well
    if (hex_count > 0 && hex_count < 16) {
        int padding_bits = 4 * (16 - hex_count);
        for (int i = 0; i < padding_bits; i++) {
            binary_out.push_back(false);
        }
    }

    return binary_out;
}

void ChangeMouseDevice(const std::pair<std::string, std::string> &device) {
    mouse = open(device.first.c_str(), O_WRONLY);
    if (mouse < 0) {
        logging::Error("could not open mouse");
        const auto error = errno;
        if (error == EACCES) {
            const std::string username = getlogin();
            logging::Error(
                "user is not in input group. please execute sudo usermod -aG input {}", username);
        } else {
            logging::Error("error code: " + std::to_string(error));
        }
        std::exit(1);
    }
    logging::Info("input device: {} ({})", device.second, device.first);
    active_device = device;
}

void MouseInit() {
    if (KernelModuleActive()) {
        mouse = open("/dev/stealthmem", O_RDWR);
        if (mouse < 0) {
            logging::Error("could not connect to kernel driver");
        }
        logging::Info("using kernel driver mouse input");
        return;
    }

    input_devices.clear();
    for (const auto &entry : std::filesystem::directory_iterator("/dev/input")) {
        if (!entry.is_character_file()) {
            continue;
        }

        const std::string event_name = entry.path().filename().string();
        if (event_name.rfind("event", 0) != 0) {
            continue;
        }

        const std::string rel_path {"/sys/class/input/" + event_name + "/device/capabilities/rel"};
        const std::vector<bool> rel_caps = DecodeCapabilities(rel_path);
        if (rel_caps.empty() || !rel_caps[REL_X] || !rel_caps[REL_Y]) {
            continue;
        }

        const std::string key_path {"/sys/class/input/" + event_name + "/device/capabilities/key"};
        const std::vector<bool> key_caps = DecodeCapabilities(key_path);
        if (key_caps.size() < BTN_LEFT || !key_caps[BTN_LEFT]) {
            continue;
        }

        const std::string name_path {"/sys/class/input/" + event_name + "/device/name"};
        std::ifstream name_file(name_path);
        if (!name_file.is_open()) {
            continue;
        }

        std::string device_name;
        std::getline(name_file, device_name);
        name_file.close();

        input_devices.push_back({entry.path(), device_name});
    }

    if (input_devices.empty()) {
        mouse = open("/dev/null", O_WRONLY);
        logging::Warning("no mouse device was found");
        std::exit(1);
    }
    ChangeMouseDevice(input_devices[0]);
}

void MouseQuit() {
    if (mouse >= 0) {
        close(mouse);
    }
}

void MouseMove(const glm::ivec2 &coords) {
    logging::Debug("mouse move: {}/{}", coords.x, coords.y);

    if (kernel) {
        mouse_move move = {.x = coords.x, .y = coords.y};
        if (ioctl(mouse, IOCTL_MOUSE_MOVE, &move) < 0) {
            logging::Warning("could not move mouse");
        }
        return;
    }

    input_event ev {};

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
    if (write(mouse, &ev, sizeof(ev)) <= 0) {
        logging::Warning("mouse write failed");
    }
}

void MouseLeftPress() {
    logging::Debug("pressed left mouse button");

    if (kernel) {
        key_press key = {.key = BTN_LEFT, .press = true};
        if (ioctl(mouse, IOCTL_KEY_PRESS, &key) < 0) {
            logging::Warning("could not press left mouse button");
        }
        return;
    }

    input_event ev {};

    // y
    ev.type = EV_KEY;
    ev.code = BTN_LEFT;
    ev.value = 1;
    write(mouse, &ev, sizeof(ev));

    // syn
    ev.type = EV_SYN;
    ev.code = SYN_REPORT;
    ev.value = 0;
    if (write(mouse, &ev, sizeof(ev)) <= 0) {
        logging::Warning("mouse write failed");
    }
}

void MouseLeftRelease() {
    logging::Debug("released left mouse button");

    if (kernel) {
        key_press key = {.key = BTN_LEFT, .press = false};
        if (ioctl(mouse, IOCTL_KEY_PRESS, &key) < 0) {
            logging::Warning("could not release left mouse button");
        }
        return;
    }

    input_event ev {};

    // y
    ev.type = EV_KEY;
    ev.code = BTN_LEFT;
    ev.value = 0;
    write(mouse, &ev, sizeof(ev));

    // syn
    ev.type = EV_SYN;
    ev.code = SYN_REPORT;
    ev.value = 0;
    if (write(mouse, &ev, sizeof(ev)) <= 0) {
        logging::Warning("mouse write failed");
    }
}

bool MouseValid() {
    if (kernel) {
        return true;
    }

    struct input_event ev {
        .time = {.tv_sec = 0, .tv_usec = 0},
        .type = EV_SYN,
        .code = SYN_REPORT,
        .value = 0,
    };
    return write(mouse, &ev, sizeof(ev)) > 0;
}
