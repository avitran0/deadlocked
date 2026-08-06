# Operating Systems

## NixOS

Add `"input"` to your user's `extraGroups` in `configuration.nix`:

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

After reboot:

```bash
git clone https://github.com/avitran0/deadlocked
cd deadlocked
direnv allow
cargo run --release
```

Everything is configured in `flake.nix` and `nix/shell.nix`.

<br>

## Fedora Atomic

```bash
grep -E '^input:' /usr/lib/group | sudo tee -a /etc/group
sudo usermod -aG input "$USER"
```

> **Restart your machine (required)**

```bash
git clone --recursive https://github.com/avitran0/deadlocked
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
Hyprland has poor X11 support for the techniques this cheat uses, not much i can do about that. May require additional tweaks.
