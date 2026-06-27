#!/bin/sh
set -e 
DIR="$(cd "$(dirname "$0")" && pwd)"

cargo build --package led --release  --target arm-unknown-linux-gnueabihf

mkdir -p $DIR/led_package/
cp "$DIR/target/arm-unknown-linux-gnueabihf/release/led" "$DIR/led_package/doom_led"
cp "$DIR"/led/*.wad "$DIR/led_package/"
cp "$DIR/led/config.toml" "$DIR/led_package/"

echo -----
echo "Game is now built in led_package/"
