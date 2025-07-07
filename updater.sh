#!/bin/bash

REPO_DIR="."
APP_PATH="$REPO_DIR/build/deadlocked"
BUILD_SCRIPT="$REPO_DIR/build.sh"

# Define colors
GREEN="\033[1;32m"
YELLOW="\033[1;33m"
RED="\033[1;31m"
BLUE="\033[1;34m"
NC="\033[0m" # No Color

print_step() {
    echo -e "${BLUE}› $1${NC}"
}

print_success() {
    echo -e "${GREEN}✔ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

print_error() {
    echo -e "${RED}✖ $1${NC}"
}

echo ""
print_step "Starting Deadlocked updater..."

cd "$REPO_DIR" || {
    print_error "Repo directory not found!"
    exit 1
}

print_step "Checking for updates..."
git fetch

LOCAL_HASH=$(git rev-parse HEAD)
REMOTE_HASH=$(git rev-parse @{u})

echo ""
print_step "Local  Hash: $LOCAL_HASH"
print_step "Remote Hash: $REMOTE_HASH"
echo ""

if [ "$LOCAL_HASH" != "$REMOTE_HASH" ]; then
    print_step "Updates found. Pulling latest changes..."
    git pull --recurse-submodules

    print_step "Recompiling project..."
    if bash "$BUILD_SCRIPT"; then
        print_success "Compilation completed successfully."
    else
        print_error "Compilation failed!"
        exit 1
    fi
else
    print_success "No updates found. Skipping compilation."
fi

echo ""
print_step "Launching Deadlocked..."
echo ""

"$APP_PATH"
