# OS-Specific setup

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

## Fedora Atomic

```bash
grep -E '^input:' /usr/lib/group | sudo tee -a /etc/group && sudo usermod -aG input $USER
# Restart your machine (required)
git clone --recursive https://github.com/avitran0/deadlocked
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
