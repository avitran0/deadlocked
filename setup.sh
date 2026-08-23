#!/usr/bin/env bash

UDEV_RULE_FILE="/etc/udev/rules.d/99-uinput.rules"
UINPUT_GROUP="uinput"
CURRENT_USER=$(whoami)

sudo modprobe uinput

echo 'KERNEL=="uinput", MODE="0660", GROUP="uinput", TAG+="uaccess"' | sudo tee "$UDEV_RULE_FILE" > /dev/null
echo "created udev file: $UDEV_RULE_FILE"

if ! getent group "$UINPUT_GROUP" > /dev/null; then
  sudo groupadd "$UINPUT_GROUP"
  echo "created group $UINPUT_GROUP"
fi

sudo usermod -aG "$UINPUT_GROUP" "$CURRENT_USER"
echo "added user $CURRENT_USER to group $UINPUT_GROUP"

sudo udevadm control --reload-rules
sudo udevadm trigger --action=add --sysname-match=uinput
sudo udevadm settle
echo "reloaded udev rules"

echo "uinput" | sudo tee /etc/modules-load.d/uinput.conf > /dev/null

if [ "$XDG_CURRENT_DESKTOP" = "Hyprland" ]; then
  echo "detected Hyprland as window manager"

  RULE='
-- deadlocked_overlay no_blur windowrule
hl.window_rule({
    match = {
        title = "^(deadlocked_overlay)$",
    },
    no_blur = true,
})'

  LUA_FILE="$HOME/.config/hypr/hyprland.lua"

  if grep -Fq 'title = "^(deadlocked_overlay)$"' "$LUA_FILE"; then
    echo "deadlocked_overlay no_blur windowrule has already been added, skipping"
  else
    echo "$RULE" >> "$LUA_FILE"
    echo "added no_blur windowrule to Hyprland"
  fi
fi
