<div align="center">

# deadlocked

[![Matrix Invite](https://img.shields.io/matrix/open-source-cs2-hacking%3Amatrix.org?style=for-the-badge\&logo=matrix\&label=Matrix)](https://matrix.to/#/%23open-source-cs2-hacking:matrix.org)
[![Discord Invite](https://img.shields.io/discord/1333541580249890949?style=for-the-badge\&logo=discord\&logoColor=white\&label=Discord)](https://discord.gg/eXjG4Ar9Sx)

[![Casual Maintenance Intended](https://casuallymaintained.tech/badge.svg)](https://casuallymaintained.tech/)

<br>

simple cs2 aimbot and esp, for linux only.

<br>

Releases are tagged `v<version>` matching the version in `Cargo.toml` (e.g. `v1.0.0`).
The built-in update checker compares against the latest release tag and will prompt when a newer version is available.

</div>

<br>

## Quick Start

> [!NOTE]
> Running NixOS, Fedora Atomic, Hyprland (Legacy .conf config)?
>
> See the [compatibility,md](compatibility.md).

Download the [latest release](https://github.com/avitran0/deadlocked/releases). Each release contains the `deadlocked` (or `deadlocked-reduced-models`) binary and `setup.sh`.

**Setup (one-time only):**

```bash
./setup.sh
```

> **Restart your machine (required)**

This creates a `uinput` group, adds your user to it, and installs a udev rule.
You only need to do this once, even when updating to newer versions.

**Run:**

`./deadlocked` or `./deadlocked-reduced-models`, depending on which version you downloaded.

The binary will refuse to start if setup hasn't been completed.
Also make sure the `uinput` kernel module is loaded.

<br>

## Build from Source

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
git clone https://github.com/avitran0/deadlocked
cd deadlocked
cargo run --release --bin deadlocked
```

<br>

## Features

### Aimbot

- Hotkey
- FOV
- Smooth
- Start bullet
- Targeting mode
- Visibility check (VPK parsing)
- Head only/whole body
- Flash check
- FOV circle

### ESP

- Hotkey
- Box
- Skeleton
- Health bar
- Armor bar
- Player name
- Weapon icon
- Player tags (helmet, defuser, bomb)
- Dropped weapons
- Bomb timer

### Triggerbot

- Activation mode
- Min/max delay
- Additional Duration
- Visibility check
- Flash check
- Scope check
- Velocity threshold
- Head only mode

### Standalone RCS

- Smoothing

### Per-Weapon Overrides

- Aimbot
- Triggerbot
- RCS

### Quake Sounds

- All 12 announcer sounds are built in, works out of the box
- First Blood, Headshot, Humiliation (knife kills)
- Multi Kills (Double Kill, Triple Kill, Multi Kill, Monster Kill)
- Kill Sprees (Killing Spree, Rampage, Dominating, Unstoppable, Godlike)
- Headshot is the lowest priority announcement, it only plays when no other
  event fired for the same kills
- Cooldown per sound keeps streaks from stacking the same announcement
- Own sound files are optional and override the built in ones
- Sequential playback queue, own kills only
- Read-only memory access, nothing is written to or injected into the game

## Web Radar

- Web radar
- Follow player
- Shareable

### Misc

- Sniper crosshair
- Bomb timer
- Grenade lineup alignment
- Web radar ([FAQ](radar.md))

### Unsafe

> [!WARNING]
> These features write to game memory and might get you banned.

- No flash (with max flash alpha)
- FOV changer
- No smoke
- Smoke color change

<br>

## FAQ

<details>
<summary>How do I set up Quake Sounds?</summary>

Just toggle **Enable Quake Sounds** in the Hitmarker tab. All 12 announcer
sounds are built into the binary, no setup required.

If you want to replace individual sounds, drop files with the same name into
`~/.config/deadlocked/sounds` (or `$XDG_CONFIG_HOME/deadlocked/sounds`). The
folder (with a template `README.txt`) is created on first start and is rescanned
automatically every few seconds. Events without a matching file keep their
built in sound.

Each event plays the first matching file, any of `wav`, `mp3`, `ogg` or `flac` works:

| File name | Event |
| --- | --- |
| `firstblood.*` | first own kill of a round (also on your first kill after joining) |
| `headshot.*` | kill by headshot, only when no other event fired for these kills |
| `humiliation.*` | kill with the knife |
| `doublekill.*` | 2 kills within the multi kill window |
| `triplekill.*` | 3 kills within the multi kill window |
| `multikill.*` | 4 kills within the multi kill window |
| `monsterkill.*` | 5+ kills within the multi kill window |
| `killingspree.*` | 3 kills without dying |
| `rampage.*` | 5 kills without dying |
| `dominating.*` | 7 kills without dying |
| `unstoppable.*` | 9 kills without dying |
| `godlike.*` | 11 kills without dying |

First Blood, Multi Kills, Kill Sprees and Humiliation take priority over the
Headshot announcement: a headshot that happens alongside them stays silent, so
fast streaks tell your ramp up instead of stacking headshot sounds. On top of
that each sound has a cooldown and plays at most once per `cooldown` seconds.

The multi kill window, the cooldown and the volume can be changed in the
`[hud.quake_sounds]` section of your config:

```toml
[hud.quake_sounds]
enabled = false
volume = 1.0
multi_kill_window = 3.0
cooldown = 5.0
```

Triggered sounds are played one after another (queue). Dying or a new round
resets sprees and multi kill chains.

</details>

<details>
<summary>Where are my configs saved?</summary>

Configs are saved in `$XDG_CONFIG_HOME` with fallback to `$HOME/.config`. Otherwise they're saved alongside the executable.

</details>

<br>

<details>
<summary>Which desktop environments and window managers are supported?</summary>

**Best support:**

- GNOME (Mutter)
- KDE (KWin)

**Good support:**

- SwayWM
- Weston

**Fair support:**

- i3
- OpenBox
- XFCE
- Hyprland (tweaks may be needed, no guarantees; see [compatibility.md](compatibility.md/#hyprland))

</details>

<br>

<details>
<summary>I'm using Hyprland and something doesn't work</summary>

Hyprland has poor X11 support for the techniques this cheat uses, not much I can do about that.
Try another WM if possible.

</details>

<br>

<details>
<summary>I'm using Gamescope and the overlay is too small</summary>

The game still thinks it's running in 16:9 resolution, so the cheat gets the wrong window resolution.
Try running the game without Gamescope.

</details>

<br>

<details>
<summary>My screen/overlay is black</summary>

Your compositor or window manager doesn't support transparency, or it's not enabled.

On KDE, go into **Display and Monitor** settings, then **Compositor**, and tick **Enable compositor on startup**.

</details>

<br>

<details>
<summary>The overlay shows but I can't click anything</summary>

The window couldn't be made click-through. This is a window manager/compositor limitation.

</details>

<br>

<details>
<summary>The overlay doesn't show up</summary>

Your window manager doesn't support positioning or resizing windows.

</details>

<br>

<details>
<summary>The overlay isn't on top of other windows</summary>

Your window manager doesn't support always-on-top windows.

</details>
