#include "process.hpp"

#include <fcntl.h>
#include <unistd.h>

#include <mithril/hex.hpp>

#ifdef __AVX2__
#include <immintrin.h>
#endif

#include <filesystem>
#include <fstream>
#include <mithril/logging.hpp>
#include <string>

#include "constants.hpp"

std::optional<i32> GetPid(const std::string &process_name) {
    for (const auto &entry : std::filesystem::directory_iterator("/proc")) {
        if (!entry.is_directory()) {
            continue;
        }

        const auto filename = entry.path().filename().string();
        const auto exe_path = "/proc/" + filename + "/exe";
        if (access(exe_path.c_str(), F_OK) != 0) {
            continue;
        }
        const auto exe_name = std::filesystem::read_symlink(exe_path).string();
        const auto pos = exe_name.rfind('/');
        // rfind returns npos on fail
        if (pos == std::string::npos) {
            continue;
        }

        const auto name = exe_name.substr(pos + 1);
        if (name == process_name) {
            return std::stoi(filename);
        }
    }

    return std::nullopt;
}

bool ValidatePid(const i32 pid) {
    return access(("/proc/" + std::to_string(pid)).c_str(), F_OK) == 0;
}

bool KernelModuleActive() { return access("/dev/stealthmem", F_OK) == 0; }

std::optional<Process> OpenProcess(const i32 pid) {
    if (!ValidatePid(pid)) {
        return std::nullopt;
    }
    if (KernelModuleActive()) {
        const i32 mem = open("/dev/stealthmem", O_RDWR);
        if (mem < 0) {
            logging::Error("could not connect to kernel driver");
            return std::nullopt;
        }
        return Process {.pid = pid, .mem = mem};
    }
    if (!flags.file_mem) {
        return Process {.pid = pid};
    }
    const i32 mem = open(("/proc/" + std::to_string(pid) + "/mem").c_str(), O_RDWR);
    if (mem < 0) {
        logging::Error("could not open /proc/{}/mem", pid);
        return std::nullopt;
    }
    return Process {.pid = pid, .mem = mem};
}

std::string Process::ReadString(const u64 address) {
    std::string value;
    value.reserve(32);
    ReadString(address, value);
    return value;
}

#ifdef __AVX2__
const __m256i zeros = _mm256_setzero_si256();
#endif
void Process::ReadString(const u64 address, std::string &value) {
#ifdef __AVX2__
    // 32 bytes at a time
    for (u64 i = address; i < address + 512; i += sizeof(__m256i)) {
        const auto chunk = Read<__m256i>(i);

        // check if any byte is zero
        const __m256i cmp = _mm256_cmpeq_epi8(chunk, zeros);
        const i32 mask = _mm256_movemask_epi8(cmp);

        // store back into char buffer
        alignas(32) char block[sizeof(__m256i)];
        _mm256_storeu_si256(reinterpret_cast<__m256i *>(block), chunk);

        if (mask != 0) {
            // at least one byte is zero, append all bytes including zero
            const u32 first_zero = __builtin_ctz(mask);
            value.append(block, first_zero);
            return;
        }
        value.append(block, sizeof(__m256i));
    }
#else
    // 8 bytes at a time
    for (u64 i = address; i < address + 512; i += sizeof(u64)) {
        const u64 chunk = Read<u64>(i);

        for (u64 offset = 0; offset < sizeof(u64); offset++) {
            const u8 byte = chunk >> offset * 8 & 0xFF;
            if (byte == 0) {
                return;
            }
            // I WANT TO FUCKING KILL MYSELF
            // DO NOT APPEND THE LAST NULL BYTE!
            // if the null byte is appended, it
            // will not match any strings because
            // of a length mismatch
            value.push_back(static_cast<char>(byte));
        }
    }
#endif
}

std::vector<u8> Process::ReadBytes(const u64 address, const u64 count) const {
    std::vector<u8> buffer(count);
    if (kernel) {
        memory_params params = {.pid = pid, .addr = address, .size = count, .buf = buffer.data()};

        if (ioctl(mem, IOCTL_READ_MEM, &params) < 0) {
            logging::Warning("could not read bytes");
            return buffer;
        }
    } else {
        const auto path = "/proc/" + std::to_string(pid) + "/mem";
        const i32 file = open(path.c_str(), O_RDONLY);
        pread(file, buffer.data(), count, static_cast<long>(address));
        close(file);
    }
    return buffer;
}

std::optional<u64> Process::GetModuleBaseAddress(const std::string &module_name) const {
    std::ifstream maps("/proc/" + std::to_string(pid) + "/maps");
    std::string line;
    while (std::getline(maps, line)) {
        if (line.rfind(module_name) == std::string::npos) {
            continue;
        }
        const size_t index = line.find_first_of('-');
        const std::string address_str = line.substr(0, index);
        u64 address = std::stoull(address_str, nullptr, 16);
        if (address == 0) {
            logging::Warning(
                "address for module {} was 0, in put string was \"{}\", extracted address was ",
                module_name, line, hex::HexString(address));
            continue;
        }

        return address;
    }

    logging::Warning("could not find address for module {}", module_name);
    return std::nullopt;
}

u64 Process::ModuleSize(const u64 module_address) {
    const u64 section_header_offset = Read<u64>(module_address + ELF_SECTION_HEADER_OFFSET);
    const u64 section_header_entry_size = Read<u16>(module_address + ELF_SECTION_HEADER_ENTRY_SIZE);
    const u64 section_header_num_entries =
        Read<u16>(module_address + ELF_SECTION_HEADER_NUM_ENTRIES);

    return section_header_offset + section_header_entry_size * section_header_num_entries;
}

std::vector<u8> Process::DumpModule(const u64 module_address) {
    const u64 module_size = ModuleSize(module_address);
    // should be 1 gb
    if (module_size == 0 || module_size > 1000000000) {
        logging::Error("could not dump module at {}", module_address);
        return {};
    }
    return ReadBytes(module_address, module_size);
}

std::optional<u64> Process::ScanPattern(
    const std::vector<u8> &pattern, const std::vector<bool> &mask, const u64 length,
    const u64 module_address) {
    const auto module = DumpModule(module_address);
    if (module.size() < 500) {
        return std::nullopt;
    }

#ifdef __AVX2__
    alignas(32) u8 pattern_vec[32] {};
    alignas(32) u8 mask_vec[32] {};

    for (u64 i = 0; i < length; i++) {
        pattern_vec[i] = pattern[i];
        mask_vec[i] = mask[i] ? 0xFF : 0x00;
    }

    const __m256i pat = _mm256_loadu_si256(reinterpret_cast<__m256i *>(&pattern_vec[0]));
    const __m256i msk_tmp = _mm256_loadu_si256(reinterpret_cast<__m256i *>(&mask_vec[0]));
    const __m256i msk = _mm256_xor_si256(msk_tmp, _mm256_set1_epi8(static_cast<char>(0xFF)));

    const u64 module_end = module.size() - 64;
    for (u64 i = 0; i < module_end; i++) {
        const __m256i candidate = _mm256_loadu_si256(reinterpret_cast<const __m256i *>(&module[i]));
        const __m256i comparison = _mm256_cmpeq_epi8(candidate, pat);
        const __m256i combined = _mm256_or_si256(comparison, msk);
        const i32 out_mask = _mm256_movemask_epi8(combined);
        if (out_mask == -1) {
            return module_address + i;
        }
    }
#else
    const u64 module_end = module.size();
    for (u64 i = 0; i < module_end - length; i++) {
        bool found = true;
        for (u64 j = 0; j < length; j++) {
            if (mask[j] && module[i + j] != pattern[j]) {
                found = false;
                break;
            }
        }
        if (found) {
            return module_address + i;
        }
    }
#endif

    logging::Warning("broken signature: {}", hex::HexStringVector(pattern));
    return std::nullopt;
}

u64 Process::GetRelativeAddress(
    const u64 instruction, const u64 offset, const u64 instruction_size) {
    const i32 rip_address = Read<i32>(instruction + offset);
    return instruction + instruction_size + rip_address;
}

std::optional<u64> Process::GetInterfaceOffset(
    const u64 module_address, const std::string &interface_name) {
    const auto create_interface = GetModuleExport(module_address, "CreateInterface");
    if (!create_interface) {
        logging::Error("could not find CreateInterface export");
        return std::nullopt;
    }

    const u64 export_address = GetRelativeAddress(*create_interface, 0x01, 0x05) + 0x10;
    if (!export_address) {
        return std::nullopt;
    }

    u64 interface_entry = Read<u64>(export_address + 0x07 + Read<u32>(export_address + 0x03));

    while (true) {
        const u64 entry_name_address = Read<u64>(interface_entry + 8);
        const std::string entry_name = ReadString(entry_name_address);
        if (entry_name.rfind(interface_name) != std::string::npos) {
            const u64 vfunc_address = Read<u64>(interface_entry);
            return Read<u32>(vfunc_address + 0x03) + vfunc_address + 0x07;
        }
        interface_entry = Read<u64>(interface_entry + 0x10);
        if (interface_entry == 0) {
            break;
        }
    }

    logging::Warning("could not find interface offset for {}", interface_name);
    return std::nullopt;
}

std::optional<u64> Process::GetModuleExport(
    const u64 module_address, const std::string &export_name) {
    constexpr u64 add = 0x18;

    const std::optional<u64> string_table_opt = GetAddressFromDynamicSection(module_address, 0x05);
    const std::optional<u64> symbol_table_opt = GetAddressFromDynamicSection(module_address, 0x06);
    if (!string_table_opt || !symbol_table_opt) {
        return std::nullopt;
    }
    const u64 string_table = *string_table_opt;
    u64 symbol_table = *symbol_table_opt;

    symbol_table += add;

    while (Read<u32>(symbol_table) != 0) {
        const u64 st_name = Read<u32>(symbol_table);
        const std::string name = ReadString(string_table + st_name);
        if (name == export_name) {
            return Read<u64>(symbol_table + 0x08) + module_address;
        }
        symbol_table += add;
    }

    logging::Warning(
        "could not find export {} in module at {}", export_name, hex::HexString(module_address));
    return std::nullopt;
}

std::optional<u64> Process::GetAddressFromDynamicSection(const u64 module_address, const u64 tag) {
    const std::optional<u64> dynamic_section_offset =
        GetSegmentFromPht(module_address, ELF_DYNAMIC_SECTION_PHT_TYPE);
    if (!dynamic_section_offset) {
        logging::Error("could not find dynamic section in loaded elf");
        return std::nullopt;
    }

    constexpr u64 register_size = 8;
    u64 address = Read<u64>(*dynamic_section_offset + 2 * register_size) + module_address;

    while (true) {
        const u64 tag_address = address;
        const u64 tag_value = Read<u64>(tag_address);

        if (tag_value == 0) {
            break;
        }
        if (tag_value == tag) {
            return Read<u64>(tag_address + register_size);
        }

        address += register_size * 2;
    }

    logging::Warning("could not find tag {} in dynamic section", tag);
    return std::nullopt;
}

std::optional<u64> Process::GetSegmentFromPht(const u64 module_address, const u64 tag) {
    const u64 first_entry = Read<u64>(module_address + ELF_PROGRAM_HEADER_OFFSET) + module_address;
    const u64 entry_size = Read<u16>(module_address + ELF_PROGRAM_HEADER_ENTRY_SIZE);

    for (u64 i = 0; i < Read<u16>(module_address + ELF_PROGRAM_HEADER_NUM_ENTRIES); i++) {
        const u64 entry = first_entry + i * entry_size;
        if (Read<u32>(entry) == tag) {
            return entry;
        }
    }

    logging::Error(
        "could not find tag {} in program header table at {}", tag, hex::HexString(module_address));
    return std::nullopt;
}

std::optional<u64> Process::GetConvar(const u64 convar_offset, const std::string &convar_name) {
    if (convar_offset == 0) {
        return std::nullopt;
    }

    const u64 objects = Read<u64>(convar_offset + 64);
    for (u64 i = 0; i < Read<u64>(convar_offset + 160); i++) {
        const u64 object = Read<u64>(objects + i * 16);
        if (object == 0) {
            break;
        }

        const u64 name_address = Read<u64>(object);
        const std::string name = ReadString(name_address);
        if (name == convar_name) {
            return object;
        }
    }

    logging::Warning("could not find convar {}", convar_name);
    return std::nullopt;
}

u64 Process::GetInterfaceFunction(const u64 interface_address, const u64 index) {
    return Read<u64>(Read<u64>(interface_address) + index * 8);
}
