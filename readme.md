# deadlocked

[![Matrix Invite](https://img.shields.io/matrix/open-source-cs2-hacking%3Amatrix.org?style=for-the-badge&logo=matrix&label=Matrix)](https://matrix.to/#/%23open-source-cs2-hacking:matrix.org)

[![Discord Invite](https://img.shields.io/discord/1333541580249890949?style=for-the-badge&logo=discord&logoColor=white&label=Discord)](https://discord.gg/eXjG4Ar9Sx)

[![Casual Maintenance Intended](https://casuallymaintained.tech/badge.svg)](https://casuallymaintained.tech/)

simple cs2 aimbot and esp, for linux only.

Releases are tagged `v<version>` matching the version in `Cargo.toml` (e.g. `v1.0.0`).
The built-in update checker compares against the latest release tag and will prompt when a newer version is available.

## Quick Start

Download the [latest release](https://github.com/avitran0/deadlocked/releases).
Each release contains the `deadlocked` binary and `setup.sh`.

**Setup (one-time only):**

```bash
./setup.sh
```
> **Restart your machine (required)**

This creates a `uinput` group, adds your user to it, and installs a udev rule.
You only need to do this once, even when updating to newer versions.

## Running

```bash
./deadlocked
```

The binary will refuse to start if setup hasn't been completed.
Also make sure the `uinput` kernel module is loaded.
Running NixOS or Fedora Atomic? See [OS-Specific Setup](os-setup.md).

## Build from Source

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
git clone https://github.com/avitran0/deadlocked
cd deadlocked
cargo run --release
```

**Run:**

```bash
./run.sh
```

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

### Misc

- Sniper crosshair
- Bomb timer

### Unsafe

> [!WARNING]
> These features write to game memory and might get you banned.

- No flash (with max flash alpha)
- FOV changer
- No smoke
- Smoke color change

## FAQ

<details>
<summary>Where are my configs saved?</summary>

Configs are saved in `$XDG_CONFIG_HOME` with fallback to `$HOME/.config`. Otherwise they're saved alongside the executable.

</details>

<br>

<details>
<summary>Which desktop environments and window managers are supported?</summary>

**Best support:**

* GNOME (Mutter)
* KDE (KWin)

**Good support:**

* SwayWM
* Weston

**Fair support:**

* i3
* OpenBox
* XFCE
* Hyprland (tweaks may be needed; no guarantees)

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
