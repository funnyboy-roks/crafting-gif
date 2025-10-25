#!/bin/sh

source=textures-256

for file in $(find "$source" -type f -printf '%P\n'); do
    mkdir -p $(dirname "textures/$file")
    magick "$source/$file" -resize 80x80 "textures/$file"
done
