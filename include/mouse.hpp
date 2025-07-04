#pragma once

#include <glm/glm.hpp>
#include <string>
#include <utility>
#include <vector>

extern std::vector<std::pair<std::string, std::string>> input_devices;
extern std::pair<std::string, std::string> active_device;

void ChangeMouseDevice(const std::pair<std::string, std::string> &device);
void MouseInit();
void MouseQuit();
void MouseMove(const glm::ivec2 &coords);
void MouseLeftPress();
void MouseLeftRelease();
bool MouseValid();
