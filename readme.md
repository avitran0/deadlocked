# deadlocked

<a href="https://matrix.to/#/%23open-source-cs2-hacking:matrix.org">
  <img src="https://img.shields.io/matrix/open-source-cs2-hacking%3Amatrix.org?style=for-the-badge&logo=matrix&label=Matrix" alt="Matrix invite" />
</a>
<a href="https://discord.gg/eXjG4Ar9Sx">
  <img src="https://img.shields.io/discord/1333541580249890949?style=for-the-badge&logo=discord&logoColor=white&label=Discord" alt="Discord invite" />
</a>

simple cs2 aimbot and esp, for linux only.

## Setup

```bash
./setup.sh
# Restart your machine (required)
git clone https://github.com/avitran0/deadlocked
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Also make sure that the `uinput` kernel module is loaded.

Running NixOS or Fedora Atomic? See [OS-Specific Setup](os-setup.md).

## Running

```bash
./run.sh
```

> [!NOTE]
> When running for the first time and on game updates,
> it will parse the map data for a fast visibility check.
> Let this run until you see all maps have been parsed.
> This will take a lot of resources, so it's best to let it run before joining a game.

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
- Spectator list

### Unsafe

> [!WARNING]
> These features write to game memory and carry ban risk.

- No flash (with max flash alpha)
- FOV changer
- No smoke
- Smoke color change

> [!CAUTION]
> VACNet 3.0 is better at detecting aimbot and wallhacks. **Do not** use aim lock. Play with a low FOV. Use visuals sparingly.

## FAQ

### Where are my configs saved?

Configs are saved in `$XDG_CONFIG_HOME` with fallback to `$HOME/.config`. Otherwise they're saved alongside the executable.

### How do I configure the radar?

See [radar.md](radar.md)

### Which desktop environments and window managers are supported?

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

**Wayland support:**

Native Wayland is supported, but requires manual compositor configuration to make the overlay work correctly (always on top and click-through).

### Hyprland Setup

For Hyprland, add the following rules to your `hyprland.conf`:

```conf
windowrulev2 = float, title:^(deadlocked_overlay)$
windowrulev2 = pin, title:^(deadlocked_overlay)$
windowrulev2 = noinput, title:^(deadlocked_overlay)$
windowrulev2 = noblur, title:^(deadlocked_overlay)$
windowrulev2 = noshadow, title:^(deadlocked_overlay)$
windowrulev2 = opaque, title:^(deadlocked_overlay)$
windowrulev2 = size 100% 100%, title:^(deadlocked_overlay)$
```

If you encounter issues, you can force X11 mode by running with the `--x11` flag.

### I'm using Gamescope and the overlay is too small

The game still thinks it's running in 16:9 resolution. This cannot be fixed.

### My screen/overlay is black

Your compositor or window manager doesn't support transparency, or it's not enabled.

On KDE, go into the `Display and Monitor` settings, then `Compositor`, and tick `Enable compositor on startup`.

### The overlay shows but I can't click anything

The window couldn't be made click-through. This is a window manager/compositor limitation.

### The overlay doesn't show up

Your window manager doesn't support positioning or resizing windows.

### The overlay isn't on top of other windows

Your window manager doesn't support always-on-top windows.
