#include "version_check.hpp"

#include <cpr/cpr.h>
#include <sys/utsname.h>

#include <cstdlib>
#include <fstream>
#include <mithril/logging.hpp>
#include <sstream>

#include "hash.hpp"
#include "json.hpp"

using json = nlohmann::json;

bool new_version = false;
SystemInfo system_info;

std::string Slurp(const std::string &file_name) {
    std::ifstream file(file_name);
    if (!file.good()) {
        return "";
    }
    std::ostringstream buf;
    buf << file.rdbuf();
    std::string content = buf.str();

    // trim trailing whitespace
    auto is_space = [](char c) { return std::isspace(static_cast<unsigned char>(c)); };
    size_t end = content.size();
    while (end > 0 && is_space(content[end - 1])) {
        --end;
    }
    content.erase(end);

    return content;
}

void VersionCheck() {
    cpr::Response res =
        cpr::Get(cpr::Url("https://api.github.com/repos/avitran0/deadlocked/branches"));
    if (res.status_code != 200) {
        logging::Warning("github api returned http {}", res.status_code);
        return;
    }
    json data;
    try {
        data = json::parse(res.text);
    } catch (json::parse_error) {
        return;
    }
    for (const auto &branch : data) {
        if (!branch.contains("name")) {
            continue;
        }
        if (branch["name"] != "main") {
            continue;
        }

        if (!branch.contains("commit")) {
            continue;
        }
        const auto &commit = branch["commit"];

        if (!commit.contains("sha")) {
            continue;
        }
        const auto &sha = commit["sha"];
        if (sha != hash) {
            new_version = true;
            logging::Info("new commit available");
        }
    }
}

std::string ReadKV(const std::string &path, const std::string &key) {
    std::ifstream f(path);
    std::string line;
    while (std::getline(f, line)) {
        if (line.rfind(key + "=", 0) == 0) {
            auto val = line.substr(key.size() + 1);
            // strip quotes if present
            if (val.size() >= 2 && val.front() == '"' && val.back() == '"')
                val = val.substr(1, val.size() - 2);
            return val;
        }
    }
    return "";
}

void GetSystemInfo() {
    SystemInfo info;

    info.hwid = Slurp("/etc/machine-id");

    info.distro = ReadKV("/etc/os-release", "PRETTY_NAME");
    if (info.distro.empty()) {
        info.distro = ReadKV("/etc/os-release", "NAME");
    }
    if (info.distro.empty()) {
        info.distro = "unknown";
    }

    char *desktop = std::getenv("XDG_CURRENT_DESKTOP");
    if (!desktop) {
        desktop = std::getenv("DESKTOP_SESSION");
    }
    if (desktop) {
        info.desktop = desktop;
    } else {
        info.desktop = "unknown";
    }

    struct utsname kernel {};
    if (uname(&kernel) == 0) {
        info.kernel = kernel.release;
    } else {
        info.kernel = "unknown";
    }

    system_info = info;

    json data {
        {"hwid", info.hwid},
        {"distro", info.distro},
        {"desktop", info.desktop},
        {"kernel", info.kernel},
    };
    const std::string info_str = data.dump();
    cpr::Post(cpr::Url("https://deadlocked.holyhades64.workers.dev"), cpr::Body(info_str));
}
