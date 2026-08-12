#!/bin/sh
docker build . -t mods
# docker build . --progress plain -t mods
docker run -v "$(pwd)/output:/output" mods:latest
file output/mods
ls -lh output
