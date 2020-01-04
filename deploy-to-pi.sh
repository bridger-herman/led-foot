# Cross-compile this project for usage on a Raspberry Pi Zero W
# (arm-unknown-linux-gnueabihf)
#
# Inspiration from: https://stackoverflow.com/a/37378989

export OPENSSL_LIB_DIR=$HOME/packages/openssl-OpenSSL_1_0_2u
export OPENSSL_INCLUDE_DIR=$HOME/packages/openssl-OpenSSL_1_0_2u/include

# Clean up from any previous binaries to reduce overall file size
cargo clean

# Allow for cross-compilation of external dependencies
export PKG_CONFIG_ALLOW_CROSS=1

# Build the project and link it
cargo build --target arm-unknown-linux-gnueabihf --release

# Tar up the whole directory...
tar -czvf /tmp/led-foot.tar.gz ../led-foot

# ... and send it to the pi
scp /tmp/led-foot.tar.gz pi@192.168.0.105:~/