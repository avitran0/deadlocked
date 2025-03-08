#include "log.hpp"

#include <iostream>

auto filter_level {LogLevel::Info};

void SetLogLevel(const LogLevel level) { filter_level = level; }

LogLevel GetLogLevel() { return filter_level; }

std::string LogLevelString(const LogLevel level) {
    switch (level) {
        case LogLevel::Debug:
            return "Debug";
        case LogLevel::Info:
            return "Info";
        case LogLevel::Warning:
            return "Warning";
        case LogLevel::Error:
            return "Error";
        case LogLevel::Off:
            return "?";
    }
    return "?";
}

void Log(const LogLevel level, const std::string &message) {
    if (level < filter_level) {
        return;
    }
    // prepare string beforehand to make this thread safe
    const std::string out = "[" + LogLevelString(level) + "] " + message + "\n";
    std::cout << out;
}
