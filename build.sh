#!/bin/bash

set -e

BUILD_TYPE=${1:-Release}

if [[ "$BUILD_TYPE" != "Release" && "$BUILD_TYPE" != "Debug" ]]; then
    echo "invalid build type: $BUILD_TYPE"
    echo "usage: $0 [Release|Debug]"
    exit 1
fi

echo "building in $BUILD_TYPE mode"

if command -v ccache &>/dev/null; then
    CCACHE_LAUNCHER_OPTS="-DCMAKE_C_COMPILER_LAUNCHER=ccache -DCMAKE_CXX_COMPILER_LAUNCHER=ccache"
else
    CCACHE_LAUNCHER_OPTS=""
fi

if command -v ninja &>/dev/null; then
    GENERATOR_OPTS="-G Ninja"
else
    GENERATOR_OPTS=""
fi

export NUM_CORES=$(nproc)

mkdir -p build
cd build

cmake $GENERATOR_OPTS \
    -DCMAKE_BUILD_TYPE=$BUILD_TYPE \
    $CCACHE_LAUNCHER_OPTS \
    -DCMAKE_CXX_FLAGS="-march=native -O3 -pipe" \
    -DCMAKE_C_FLAGS="-march=native -O3 -pipe" \
    ..

cmake --build . -j$NUM_CORES

if command -v ccache &>/dev/null; then
    ccache --show-stats
fi
