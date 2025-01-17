# deadlocked

![downloads](https://img.shields.io/github/downloads/avitran0/deadlocked/total?color=blue)

> [!WARNING]
> i have exams right now, will work more on this when those are over.
> aim lock will be fixed soon!

a very simple cs2 aimbot, for linux only

deadlock support will happen once that gets a native linux client

## setup

- add your user to the `input` group: `sudo usermod -aG input USERNAME` (replace USERNAME with your actual username)
- restart your machine (this will ***not*** work without a restart!)

## running

- if you got the source code from github, run with cargo: `cargo run --release`
- if you got a standalone binary, just run that

## troubleshooting

- x11 often has problems with transparent framebuffers, so the overlay window might be black. you can try running wayland, if your distro supports it, and if this still does not work, you can try the `cpp` branch. this is a rewrite of the esp part in c++, which uses glfw and might work better. this currently has not aimbot support.
- if you only want an aimbot: the aptly titled `aimbot` branch does this, and performance might be better, especially if your computer is not as powerful.

## images

### ingame demo

![demo](docs/demo_ingame.png)

### aimbot settings

![demo](docs/demo_aimbot.png)

### visuals settings

![demo](docs/demo_visuals.png)

## security

- should be "undetectable", as far as user-space externals go
- completely in user-space
- does not write anything to game memory
- does not need root permissions
