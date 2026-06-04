#!/usr/bin/env bash

UDEV_RULE_FILE="/etc/udev/rules.d/99-uinput.rules"
UINPUT_GROUP="uinput"
CURRENT_USER=$(whoami)

git config core.hooksPath .hooks

echo 'KERNEL=="uinput", MODE="0660", GROUP="uinput"' | sudo tee "$UDEV_RULE_FILE" > /dev/null
echo "created udev file: $UDEV_RULE_FILE"

if ! getent group "$UINPUT_GROUP" > /dev/null; then
    sudo groupadd "$UINPUT_GROUP"
    echo "created group $UINPUT_GROUP"
fi

sudo usermod -aG "$UINPUT_GROUP" "$CURRENT_USER"
echo "added user $CURRENT_USER to group $UINPUT_GROUP"

sudo udevadm control --reload-rules
sudo udevadm trigger
echo "reloaded udev rules"

if [ "$XDG_CURRENT_DESKTOP" = "Hyprland" ]; then
    echo "detected Hyprland as window manager"

    RULE="windowrule = no_blur 1, match:title ^(deadlocked_overlay)$"
    CONF_FILE="$HOME/.config/hypr/hyprland.conf"

    if grep -Fxq "$RULE" "$CONF_FILE"; then
        echo "deadlocked_overlay windowrule has already been added, skipping"
    else
        echo "$RULE" >> "$CONF_FILE"
        echo "added windowrule to Hyprland"
    fi
fi
