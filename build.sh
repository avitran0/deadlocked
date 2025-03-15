#!/bin/bash

set -e

BUILD_TYPE=${1:-Release}

if [[ "$BUILD_TYPE" != "Release" && "$BUILD_TYPE" != "Debug" ]]; then
    echo "Invalid build type: $BUILD_TYPE"
    echo "Usage: $0 [Release|Debug]"
    exit 1
fi

echo "Building in $BUILD_TYPE mode..."

export CC="ccache gcc"
export CXX="ccache g++"
export CMAKE_GENERATOR="Ninja"
export NUM_CORES=$(nproc)


mkdir -p build
cd build

cmake -G Ninja \
    -DCMAKE_BUILD_TYPE=$BUILD_TYPE \
    -DCMAKE_C_COMPILER_LAUNCHER=ccache \
    -DCMAKE_CXX_COMPILER_LAUNCHER=ccache \
    -DCMAKE_CXX_FLAGS="-march=native -O3 -pipe" \
    -DCMAKE_C_FLAGS="-march=native -O3 -pipe" \
    ..

cmake --build . -j$NUM_CORES

ccache --show-stats
