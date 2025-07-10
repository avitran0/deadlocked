#include "hex.hpp"

#include <iomanip>

std::string hex::HexStringVector(std::vector<u8> vec) {
    std::ostringstream oss;
    oss << "[" << std::hex << std::setw(2);
    for (std::size_t i = 0; i < vec.size(); i++) {
        const u8 &c = vec[i];
        if (i != 0) {
            oss << ", ";
        }
        oss << "0x" << std::setfill('0') << static_cast<u32>(c);
    }
    oss << "]";
    return oss.str();
}
