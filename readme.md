# deadlocked

![downloads](https://img.shields.io/github/downloads/avitran0/deadlocked/total?color=blue) [![foss cs2/deadlock hacking](https://badgen.net/discord/members/eXjG4Ar9Sx)](https://discord.gg/eXjG4Ar9Sx)

> [!CAUTION]
> vacnet 3.0 seems to be better at detecting aimbot and wallhacks, so **do not** use aim lock, and play with a low fov to avoid bans. the default configuration should be a good starting point.

a very simple cs2 aimbot, for linux only

deadlock support will happen once that gets a native linux client

> [!WARNING]
> using the glow feature might get you banned. use with caution.

## setup

- add your user to the `input` group: `sudo usermod -aG input USERNAME` (replace USERNAME with your actual username)
- restart your machine (this will **_not_** work without a restart!)

## running

- if you got the source code from github, run with cargo: `cargo run --release`
- if you got a standalone binary, just run that
