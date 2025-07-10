#pragma once

#include <ios>
#include <sstream>
#include <string>
#include <vector>

#include "types.hpp"

namespace hex {
    template <typename T>
    std::string HexString(T value) {
        static_assert(std::is_integral<T>::value, "T must be an integral type");
        std::ostringstream ss;
        ss << std::hex << "0x" << value;
        return ss.str();
    }

    std::string HexStringVector(std::vector<u8> vec);
}  // namespace hex
