# deadlocked

<a href="https://www.buymeacoffee.com/avitrano">
<img src="https://cdn.buymeacoffee.com/buttons/default-orange.png" width="130" height="30" alt="buy me a coffee" />
</a>

![downloads](https://img.shields.io/github/downloads/avitran0/deadlocked/total?color=blue)
[![foss cs2/deadlock hacking](https://badgen.net/discord/members/eXjG4Ar9Sx)](https://discord.gg/eXjG4Ar9Sx)

simple cs2 aimbot and esp, for linux only.

## features

- aimbot
  - fov
  - smoothing (with jitter)
  - aim lock
  - visibility check
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
- triggerbot
  - min/max delay
  - hotkey
  - visibility check
  - flash check
  - scope check
- standalone rcs
- misc
  - sniper crosshair
- unsafe
  - noflash
    - max flash alpha
  - fov changer

> [!WARNING]
> the features in the unsafe tab are there for a reason.
> do not use them unless you are fine with risking a ban.
> they write to game memory.

> [!CAUTION]
> vacnet 3.0 seems to be better at detecting aimbot and wallhacks, so **do not** use aim lock,
> and play with a low fov to avoid bans. use visuals sparingly.

## setup

- add your user to the `input` group: `sudo usermod -aG input USERNAME` (replace USERNAME with your actual username)
- restart your machine (this will **_not_** work without a restart!)
- clone the repository: `git clone --recursive https://github.com/avitran0/deadlocked`
- install cmake ,a c++17 compiler (gcc 7+ or clang 4+), ninja and ccache 

## running

- execute the run script: `./run.sh`

## flags

- `--file-mem`: if process_vm_readv does not work/you get an error that it cannot get the resource offset
- `--scale=X`: override gui scale
- `--no-visuals`: disable visuals completely, if there are problems with the overlay
- `--verbose` or `-v`: increase log level by one
- `--silent`: decrease log level by one

## faq

### what desktop environments and window managers are supported?

it is tested on GNOME with Mutter, and KDE with KWin.
support for other (especially tiling) window managers is not guaranteed.
if in doubt, use either one of the two just listed.

### the overlay window/my screen is black

your compositor or window manager does not support transparency, or it is not enabled.

### the overlay shows up, but i cannot click on anything

the window could not be made click-through, which might be because of window manager/compositor support.

### the overlay does not show up

you window manager does not support positioning or resizing the window.

### the overlay is not on top of other windows

your window manager does not support always on top windows.
