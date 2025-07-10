#pragma once

#include <csignal>
#include <string>
#include <vector>

#include "types.hpp"

namespace stacktrace {
    // skips a number of frames
    void SkipFrames(i32 frames);
    // prints a stack trace on receiving a signal
    void SignalHandler(i32 signal);
    std::vector<std::string> Stacktrace();

    void SetupSignalHandlers(sig_t handler);
}  // namespace stacktrace
