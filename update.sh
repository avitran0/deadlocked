#!/bin/bash

REPO_DIR="."
APP_PATH="$REPO_DIR/build/deadlocked"
BUILD_SCRIPT="$REPO_DIR/build.sh"

# define colors
GREEN="\033[1;32m"
YELLOW="\033[1;33m"
RED="\033[1;31m"
BLUE="\033[1;34m"
NC="\033[0m" # reset color

print_step() {
    echo -e "${BLUE}$1${NC}"
}

print_success() {
    echo -e "${GREEN}$1${NC}"
}

print_warning() {
    echo -e "${YELLOW}$1${NC}"
}

print_error() {
    echo -e "${RED}$1${NC}"
}

cd "$REPO_DIR" || {
    print_error "repo directory not found!"
    exit 1
}

print_step "checking for updates..."
git fetch

LOCAL_HASH=$(git rev-parse HEAD)
REMOTE_HASH=$(git rev-parse @{u})

echo ""
print_step "local  hash: $LOCAL_HASH"
print_step "remote hash: $REMOTE_HASH"
echo ""

if [ "$LOCAL_HASH" != "$REMOTE_HASH" ]; then
    print_step "update(s) found, pulling latest changes..."
    git pull --recurse-submodules

    print_step "recompiling project..."
    bash "$BUILD_SCRIPT"
else
    print_success "no updates found."
fi
