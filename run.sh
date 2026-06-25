#!/usr/bin/env bash

fail() {
	echo "Not a git repository (or any of the parent directories): .git.
Do NOT download the repository as a zip file from github.com!
Please download deadlocked by cloning the Git repository: 'git clone https://github.com/avitran0/deadlocked'"
	exit 1
}

[[ -d '.git' ]] || fail

radar() {
    cd radar/client
    npm run build
    cd ../../
    mkdir -p radar/server/assets
    cp -r radar/client/dist/* radar/server/assets/
    cd radar/server
    cargo run --bin server --release
}

cheat() {
    cargo run --bin deadlocked --release
}

# git pull

if [[ $1 == "radar" ]]; then
    radar
else
    cheat
fi
