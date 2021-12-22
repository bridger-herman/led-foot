//! Manager for Belkin WeMo devices
//!
//! Uses Python library ouimeaux (https://github.com/iancmcc/ouimeaux) which is
//! technically deprecated but works for these purposes.
//!
//! Assumes that you have the `wemo` command installed (pip3 install ouimeaux)
//! and accessible on the PATH.
//!
//! NOTE that operation of these devices can be fairly slow, especially when
//! executed on a Raspberry Pi.

use std::process::Command;

const COMMANDS: [&str; 3] = ["on", "off", "toggle"];
const ERROR_TEXT: &str = "Wemo command is not installed! Please install it with `pip3 install ouimeaux`.";

// Deliberately does not store state for Wemos because their state is
// temperamental and can be controlled by other apps which do not sync well
// without polling
pub struct WemoManager {}

impl WemoManager {
    pub fn new() -> Self {
        // Check to see if ouimeaux is installed
        let output = Command::new("wemo")
            .arg("--help")
            .output()
            .expect(ERROR_TEXT);
        if !output.status.success() {
            panic!("{}", ERROR_TEXT);
        }
        Self {}
    }

    pub fn send_wemo_command(&self, wemo: &str, command: &str) {
        if let Some(_idx) = COMMANDS.iter().position(|&cmd| cmd == command) {
            let output = Command::new("wemo")
                .arg("switch")
                .arg(wemo)
                .arg(command)
                .output()
                .expect(ERROR_TEXT);
            if !output.status.success() {
                error!("Failed to send command to WeMo device {}", wemo);
            } else {
                debug!("Sent command to WeMo device {}", wemo);
            }
        } else {
            error!("{} is not a valid wemo command.", command);
        }
    }
}
