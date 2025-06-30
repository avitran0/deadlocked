#pragma once

#include <mithril/types.hpp>
#include <string>

struct SystemInfo {
    std::string hwid;
    std::string distro;
    std::string desktop;
    std::string kernel;
};

void VersionCheck();
void GetSystemInfo();
