#!/usr/bin/env sh
# Automated screenshot: ./shot.sh name frames [extra user args...]
cd "$(dirname "$0")"
N=$1; F=$2; shift 2
mkdir -p screenshots
timeout 300 xvfb-run -a -s "-screen 0 1600x900x24" ${GODOT:-$HOME/tools/Godot_v4.4.1-stable_linux.x86_64} --path . --rendering-driver opengl3 --resolution 1600x900 -- --screenshot="$PWD/screenshots/$N.png" --frames="$F" "$@" 2>&1 | grep -E "SCRIPT ERROR|ERROR: .*(gd|Parse)|at: .*\.gd|saved screenshot" 
