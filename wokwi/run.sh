#!/bin/bash
set -e
if [ `whoami` == "esp" ]; then
path="/home/esp/workspace"
else
path="/workspace/esp-rs-devcontainer"
fi
echo Building and running Wokwi simulation for: $CURRENT_PROJECT 
cd $CURRENT_PROJECT
cargo espflash --release save-image app.bin
cd $path
python3 $path/wokwi/wokwi-server.py