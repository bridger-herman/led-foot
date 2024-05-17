# LED automation
RGB+White LEDs controlled by an Arduino (Mega 2560), and a web server that integrates with [Home Assistant](https://homeassistant.io)

Server is written in Rust, using the Actix framework.

## Installation

0. Download and install dependencies:
- Rust Compiler: follow the instructions on <https://rustup.rs>

1. Clone this repo to a location of your choice

```
git clone https://github.com/bridger-herman/led-foot.git
```

2. Build the project (release mode, optimized)

```
cd led-foot
cargo build --release
```

3. Test and make sure the project runs correctly. It can be a pain
to have all of actix's log output in some of these cases, see the below code
block for enabling ONLY LED Foot's output.

```
cargo run --release
RUST_LOG=debug cargo run --release
RUST_LOG=none,led_foot=trace cargo run --release
```

4.  (optional) to run on startup, install the `systemd` service. NOTE, you may
need to adjust the `led-foot.service` file for your setup (paths are hardcoded)

```
sudo cp ./systemd/led-foot.service /etc/systemd/system
sudo systemctl enable led-foot.service
sudo systemctl start led-foot.service

# to get log information if it failed, check
sudo journalctl -u led-foot.service
```

5. (optional) you may need to adjust the permissions of the Arduino connected
via USB (usually `/dev/ttyACM0` on Linux):

```
sudo chmod a+rw /dev/ttyACM0
```

6. (optional) set up with Home Assistant

  1.  Copy the ha-integration to your home assistant custom components folder (e.g., in `<config folder>/custom_components/`).
  2. In the Home Assistant GUI, add a new integration & search for "LED Foot"
  3. Add the integration!
  4. Check the log if anything goes wrong.

---

Everything below here is all deprecated now that we're just running this on a regular x86_64 server.

## Part 1: Set up cross compiler for the target

Tested on Ubuntu 22.04, with the Raspberry Pi Zero W as a target (Arm v6 architecture with hardware floating point).

Run the following commands

- `sudo apt install libssl-dev` (for build on regular x86_64)
- `sudo apt install gcc-arm-linux-gnueabihf`
- `rustup target add arm-unknown-linux-gnueabihf`


## Part 2: Set up Cargo/Rust config

Add the following to your local `$HOME/.cargo/config`:

```
[target.arm-unknown-linux-gnueabihf]
linker = "arm-linux-gnueabihf-gcc"
```


## Part 3: Build the project for the other target


Build the project and link it (note, this may use the wrong version of `glibc`...)
```
cargo build --target arm-unknown-linux-gnueabihf --release
```

Then tar up the necessary files (zip up everything that should be copied over to the target machine)

```
rm -rf /tmp/led-foot /tmp/led-foot.tar.gz
mkdir -p /tmp/led-foot

cp -r led-foot-sequences index.html reinitialize_serial.py static systemd target/arm-unknown-linux-gnueabihf/release/led-foot /tmp/led-foot

cd /tmp
tar -czvf /tmp/led-foot.tar.gz ./led-foot
cd -
```


Finally, send the tarball to the target machine:

```
scp /tmp/led-foot.tar.gz pi@<the IP address>:~/
```


## Part 4: Setup on the target system


Stop the LED Foot service and make a backup of the old code that's running

```
sudo systemctl stop led-foot
mv led-foot led-foot.bak.<date>
```

Untar the new files

```
tar -xzvf led-foot.tar.gz
```

Restart the LED Foot systemd service and check running status

```
sudo systemctl start led-foot
sudo systemctl status led-foot
```





### Below this is old news

retaining in case stuff changes in the future and we need to build from source again...

- A version of the `arm-linux-gnueabihf-gcc` compiler
  - This can be found inside in the [Raspberry Pi Tools
  Repository](https://github.com/raspberrypi/tools)
  - OR, with [this AUR
    package](https://aur.archlinux.org/packages/arm-linux-gnueabihf-gcc-linaro-bin/)
    (currently not working?)
  - Add the compiler to PATH:
    - `export
      PATH=$PATH:/<path-to-tools-repo>/arm-bcm2708/gcc-linaro-arm-linux-gnueabihf-raspbian/bin/`
    - OR, `export
      PATH=$PATH:/<path-to-tools-repo>/arm-bcm2708/arm-rpi-4.9.3-linux-gnueabihf/bin`
- At one point, building OpenSSL was necessary (seems to not be anymore). Build
OpenSSL (inspiration from [this StackOverflow
post](https://stackoverflow.com/a/37378989))
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
  `libssl.so` files into `/usr/lib/arm-linux-gnueabihf` on the Pi to make it work

- Additionally, you may need to copy libraries like `libgcc_s` and `libgcc` into
the cargo lib directory
`$HOME/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/arm-unknown-linux-gnueabihf/lib`.
These can be found inside a [crosstool-ng](http://crosstool-ng.github.io/)
install, or in the [Raspberry Pi Tools
Repository](https://github.com/raspberrypi/tools)


## Rust compilation

- Add the location of `arm-linux-gnueabihf-gcc` (from crosstool-ng or
  Raspberry Pi tools) to the PATH so it can be located by `cargo`:

```
# Crosstool NG
export PATH=${PATH}:$HOME/x-tools/armv6-rpi-linux-gnueabihf/bin

# RPi Tools
export PATH=$PATH:/<path-to-tools-repo>/arm-bcm2708/gcc-linaro-arm-linux-gnueabihf-raspbian/bin/

# RPi Tools (most current)
export PATH=$PATH:/<path-to-tools-repo>/arm-bcm2708/arm-rpi-4.9.3-linux-gnueabihf/bin`
```

- Use the provided `deploy-to-pi.sh` to compile the project binaries for the
  Raspberry Pi, and send them over `scp` to the device.
