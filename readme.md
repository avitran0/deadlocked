# deadlocked

![downloads](https://img.shields.io/github/downloads/avitran0/deadlocked/total?color=blue) [![foss cs2/deadlock hacking](https://badgen.net/discord/members/eXjG4Ar9Sx)](https://discord.gg/eXjG4Ar9Sx)

> [!CAUTION]
> vacnet 3.0 seems to be better at detecting aimbot and wallhacks, so **do not** use aim lock, and play with a low fov to avoid bans. the default configuration should be a good starting point.

simple cs2 aimbot, for linux only.

if you want esp, try either the esp branch, and if that does not work the cpp branch.

## features

- aimbot
  - fov
  - smoothing (with jitter)
  - aim lock
  - visibility check
  - head only/whole body
- rcs
- triggerbot
  - min and max delay in milliseconds
- unsafe
  - glow
  - noflash
    - max flash alpha

## setup

- add your user to the `input` group: `sudo usermod -aG input USERNAME` (replace USERNAME with your actual username)
- restart your machine (this will **_not_** work without a restart!)

## running

- if you got the source code from github, run with cargo: `cargo run --release`
- if you got a standalone binary, just run that

## headless mode

if the gui has problems, you can run this in headless mode. it will watch the config file for changes and update on the fly. to run like this, use the flag `--headless`
