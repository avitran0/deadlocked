#include <csignal>
#include <mithril/logging.hpp>
#include <mithril/stacktrace.hpp>

#include "globals.hpp"
#include "gui.hpp"
#include "mouse.hpp"

int main(const int argc, const char *argv[]) {
    signal(SIGILL, stacktrace::SignalHandler);
    signal(SIGABRT, stacktrace::SignalHandler);
    signal(SIGSEGV, stacktrace::SignalHandler);

    for (int i = 0; i < argc; i++) {
        const std::string arg {argv[i]};
        if (arg == "--file-mem") {
            logging::Info("reading memory from file");
            flags.file_mem = true;
        } else if (arg.find("--scale=") != std::string::npos) {
            misc_info.gui_scale = std::stof(arg.substr(8));
        } else if (arg == "--no-visuals") {
            logging::Info("disabling visuals");
            flags.no_visuals = true;
        } else if (arg == "--verbose" || arg == "-v") {
            const logging::Level level = logging::GetLevel();
            if (level > logging::Level::Debug) {
                logging::SetLevel(static_cast<logging::Level>(static_cast<i32>(level) - 1));
                logging::Info("log level set to: {}", logging::LevelName(logging::GetLevel()));
            }
        } else if (arg == "--silent") {
            const logging::Level level = logging::GetLevel();
            if (level < logging::Level::Off) {
                logging::SetLevel(static_cast<logging::Level>(static_cast<i32>(level) + 1));
                logging::Info("log level set to: {}", logging::LevelName(logging::GetLevel()));
            }
        }
    }
    logging::Info("build time: {} {}", std::string(__DATE__), std::string(__TIME__));
    MouseInit();
    Gui();
}
