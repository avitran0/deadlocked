# Operating Systems

## NixOS

Nix flake files (`flake.nix`, `nix/shell.nix`) are **not** in this repository.
They were removed because nobody was maintaining them. `direnv allow` will not
set up a dev shell unless you add those files yourself.

Add `"input"` (and `"uinput"` if you create that group) to your user's
`extraGroups` in `configuration.nix`:

```nix
users.users.yourname = {
  isNormalUser = true;
  extraGroups = [ "wheel" "input" ];
};
```

Then rebuild and reboot:

```bash
sudo nixos-rebuild switch
sudo reboot
```

After reboot, install a Rust toolchain (for example with rustup), clone the
repo, and build with Cargo:

```bash
git clone https://github.com/avitran0/deadlocked
cd deadlocked
# rustc 1.85+ is required (edition 2024)
cargo run --release
```

<br>

## Fedora Atomic

```bash
grep -E '^input:' /usr/lib/group | sudo tee -a /etc/group
sudo usermod -aG input "$USER"
```

> **Restart your machine (required)**

```bash
git clone https://github.com/avitran0/deadlocked
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

<br>
<br>

# Window Managers

## Hyprland

The setup script automatically adds the required `no_blur` window rule for users running the Lua configuration format.

```lua
hl.window_rule({
	match = {
		title = "^(deadlocked_overlay)$",
	},
	no_blur = true,
})

```

If you're using the legacy .conf configuration (_Deprecated as of Hyprland 0.55, but still supported_) , add the following rule manually to `hyprland.conf`:

```conf
windowrule = no_blur 1, match:title ^(deadlocked_overlay)$
```

You should see **two** XWayland windows: `deadlocked` (settings GUI) and
`deadlocked_overlay`. If only the overlay appears, the settings window was not
mapped; that was a v1.1.0 regression and is fixed by presenting the GUI for a
few frames even when unfocused.

Hyprland has poor X11 support for the overlay technique this project uses.
Tweaks may still be needed.

## niri

niri has known issues with overlay window positioning. The settings GUI should
still open. Overlay alignment on niri is a compositor limitation and is not
fully supported.
