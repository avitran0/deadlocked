#!/bin/bash
if [ ! -d "png" ]; then
  mkdir -p "png"
fi
for file in svg/*.svg; do
    inkscape "$file" --export-filename="png/${file%.svg}.png" --export-height=256
done
