#!/bin/bash
# Cross-compile this project for usage on a Raspberry Pi Zero W
# (arm-unknown-linux-gnueabihf)
#
# Inspiration from: https://stackoverflow.com/a/37378989

# Locate OpenSSL lib (must build OpenSSL first)
export OPENSSL_LIB_DIR=$HOME/packages/openssl-OpenSSL_1_1_1h
export OPENSSL_INCLUDE_DIR=$HOME/packages/openssl-OpenSSL_1_1_1h/include

# Tell Rust where arm-linux-gnueabihf-gcc is
# export PATH=$PATH:$HOME/GitHub/tools/arm-bcm2708/gcc-linaro-arm-linux-gnueabihf-raspbian/bin
export PATH=$PATH:$HOME/GitHub/tools

# Allow for cross-compilation of external dependencies
export PKG_CONFIG_ALLOW_CROSS=1

# Build the project and link it
cargo build --target arm-unknown-linux-gnueabihf --release

# Tar up the necessary files...
rm -rf /tmp/led-foot /tmp/led-foot.tar.gz
mkdir -p /tmp/led-foot

cp -r led-foot-sequences index.html static systemd target/arm-unknown-linux-gnueabihf/release/led-foot /tmp/led-foot

cd /tmp
tar -czvf /tmp/led-foot.tar.gz ./led-foot
cd -

# ... and send them to the pi
scp /tmp/led-foot.tar.gz pi@192.168.0.105:~/
