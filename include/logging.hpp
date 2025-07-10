#pragma once

#include <iostream>
#include <sstream>
#include <string>

#define COLOR_RESET "\033[0m"
#define COLOR_YELLOW "\033[33m"
#define COLOR_RED "\033[31m"

namespace logging {
    enum class Level {
        Debug,
        Info,
        Warning,
        Error,
        Off,
    };
}

extern logging::Level log_level;

namespace logging {
    std::string LevelName(Level level);
    void LogImpl(std::ostringstream &oss, const std::string &fmt, std::size_t pos);
    std::size_t CountPlaceholders(const std::string &format);

    // recursively process one arg per {}
    template <typename T, typename... Args>
    void LogImpl(
        std::ostringstream &oss, const std::string &format, std::size_t pos, T value,
        Args... args) {
        std::size_t placeholder_pos = format.find("{}", pos);
        if (placeholder_pos == std::string::npos) {
            throw std::runtime_error("Not enough '{}' placeholders in format string");
        }
        oss << format.substr(pos, placeholder_pos - pos) << value;
        LogImpl(oss, format, placeholder_pos + 2, args...);
    }

    template <typename... Args>
    std::string Log(const std::string &format, Args &&...args) {
        std::size_t num_placeholders = CountPlaceholders(format);
        std::size_t num_args = sizeof...(args);

        if (num_placeholders != num_args) {
            throw std::runtime_error(
                "Number of '{}' placeholders does not match number of arguments");
        }

        std::ostringstream oss;
        LogImpl(oss, format, 0, args...);
        return oss.str() + "\n";
    }

    template <typename... Args>
    void Debug(const std::string &format, Args &&...args) {
        if (log_level > Level::Debug) {
            return;
        }
        const std::string formatted = "[DEBUG] " + Log(format, args...);
        std::cout << formatted;
    }

    template <typename... Args>
    void Info(const std::string &format, Args &&...args) {
        if (log_level > Level::Info) {
            return;
        }
        const std::string formatted = "[INFO] " + Log(format, args...);
        std::cout << formatted;
    }

    template <typename... Args>
    void Warning(const std::string &format, Args &&...args) {
        if (log_level > Level::Warning) {
            return;
        }
        const std::string formatted = "[WARN] " + Log(format, args...);
        std::cout << COLOR_YELLOW << formatted << COLOR_RESET;
    }

    template <typename... Args>
    void Error(const std::string &format, Args &&...args) {
        if (log_level > Level::Error) {
            return;
        }
        const std::string formatted = "[ERROR] " + Log(format, args...);
        std::cout << COLOR_RED << formatted << COLOR_RESET;
    }

    void SetLevel(Level level);
    Level GetLevel();
}  // namespace logging
