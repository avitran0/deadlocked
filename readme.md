# deadlocked

![downloads](https://img.shields.io/github/downloads/avitran0/deadlocked/total?color=blue)
[![foss cs2 hacking](https://badgen.net/discord/members/eXjG4Ar9Sx)](https://discord.gg/eXjG4Ar9Sx)

simple cs2 aimbot and esp, for linux only.

## features

- aimbot
  - fov
  - smoothing
  - visibility check (vpk parsing)
  - head only/whole body
  - flash check
  - fov circle
- esp
  - box
  - skeleton
  - health bar
  - armor bar
  - player name
  - weapon name
  - player tags (helmet, defuser, bomb)
  - dropped weapons
  - bomb timer
- triggerbot
  - min/max delay
  - visibility check
  - flash check
  - scope check
  - velocity threshold
  - head only mode
- standalone rcs
  - smoothing
- aimbot, triggerbot and rcs per-weapon overrides
- misc
  - sniper crosshair
- unsafe
  - bhop (auto-jump)
  - strafe helper (auto A/D based on mouse movement)
  - no flash
    - max flash alpha
  - fov changer
  - no smoke
  - smoke color change

> [!WARNING]
> the features in the unsafe tab are there for a reason.
> do not use them unless you are fine with risking a ban.
> they write to game memory.

> [!CAUTION]
> vacnet 3.0 seems to be better at detecting aimbot and wallhacks, so **do not** use aim lock,
> and play with a low fov to avoid bans. use visuals sparingly.

## setup

- add your user to the `input` group: `sudo usermod -aG input $(whoami)`
- restart your machine (this will **_not_** work without a restart!)
- clone the repository: `git clone --recursive https://github.com/avitran0/deadlocked`
- install rust from `https://rustup.rs/`

### optional: for bhop/strafe helper features

if the bhop/strafe doesn't work, you may need to load the uinput kernel module:
- `sudo modprobe uinput`
- to load it automatically on boot: `echo "uinput" | sudo tee /etc/modules-load.d/uinput.conf`

## running

- `cargo run --release`

## faq

### where are my configs at?

they are saved in `$XDG_CONFIG_HOME`, if that is set.
that usually defaults to `$HOME/.config`.
if not, they are saved alongside the executable.

### radar

see [radar.md](radar.md)

### what desktop environments and window managers are supported?

it is tested on GNOME with Mutter, KDE with KWin, and SwayWM.
support for other (especially tiling) window managers is not guaranteed.
if in doubt, use either GNOME or KDE.

### i'm using hyprland and something does not work

too bad, hyprland has bad support for the x11 shenanigans this cheat tries to do.
this is nothing i can fix, and i doubt hyprland will improve its x11 support.

### i'm using gamescope to stretch the window, and the overlay is too small

this is because the game still thinks it's running in a 16/9 resolution.
i can't really fix that, unfortunately.

### the overlay window/my screen is black

your compositor or window manager does not support transparency, or it is not enabled.

### the overlay shows up, but i cannot click on anything

the window could not be made click-through, which might be because of window manager/compositor support.

### the overlay does not show up

you window manager does not support positioning or resizing the window.

### the overlay is not on top of other windows

your window manager does not support always on top windows.

### the strafe helper/auto-strafe is not working

make sure you've loaded the uinput kernel module: `sudo modprobe uinput`
check if the virtual keyboard was created by looking for this log message when starting:
`[INFO] created virtual keyboard device`
if you see warnings about uinput, ensure your user is in the `input` group and the uinput module is loaded.
