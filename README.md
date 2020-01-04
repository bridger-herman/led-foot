# LED automation
RGB+White LEDs controlled by a Raspberry Pi (Zero W) and an Arduino (Mega 2560)

Server is written in Rust, using the Actix framework.

## Cross-Compilation Setup

Tested on Arch Linux, with the Raspberry Pi Zero W as a target

### Tools needed
- A version of the `arm-linux-gnueabihf-gcc` compiler
    - Can be installed with [this AUR package](https://aur.archlinux.org/packages/arm-linux-gnueabihf-gcc-linaro-bin/). Clone the repository then run `makepkg -si`.
    - Also can use [crosstool-ng](https://github.com/crosstool-ng/crosstool-ng)

- Build OpenSSL (inspiration from [this StackOverflow post](https://stackoverflow.com/a/37378989))
    - Download a release from [the releases page](https://github.com/openssl/openssl/releases) and extract it
    - Prepare OpenSSL:

        ```
        export MACHINE=armv6
        export ARCH=arm
        export CC=arm-linux-gnueabihf-gcc
        ```

    - Compile OpenSSL

        ```
        cd <OpenSSL directory that was just extracted>
        ./config shared
        make -j
        cd -
        ```

- If the versions of OpenSSL are mismatched, you may need to copy some
  `libssl.so` files into `/usr/lib/arm-linux-gnueabihf` to make it work

- Additionally, you may need to copy libraries like `libgcc_s` and `libgcc`
  into the cargo lib directory
  `$HOME/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/arm-unknown-linux-gnueabihf/lib`.
  These can be found inside the crosstool-ng install, or in the [Raspberry Pi
  Tools Repository](https://github.com/raspberrypi/tools)


## Rust configuration

Add the following to your local `$HOME/.cargo/config`:

```
[target.arm-unknown-linux-gnueabihf]
linker = "arm-linux-gnueabihf-gcc"
```

## Rust compilation

Use the provided `deploy-to-pi.sh` to compile the project binaries for the
Raspberry Pi, and send them over `scp` to the device.
