#pragma once

#include <mithril/types.hpp>
#include <string>

struct SystemInfo {
    std::string hwid = "unknown";
    std::string distro = "unknown";
    std::string desktop = "unknown";
    std::string kernel = "unknown";
};

void VersionCheck();
void GetSystemInfo();
