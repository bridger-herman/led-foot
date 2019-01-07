use std::io::{Read, Write};
use std::time::Duration;

use serial::{SerialPort, SystemPort};

use crate::color::Color;

/// Controls the RGBW LEDs
pub struct LedSystem {
    pub current_color: Color,
    pub serial: SystemPort,
}

impl LedSystem {
    pub fn new(tty_name: &str) -> Self {
        let mut serial = serial::open(tty_name).unwrap();
        serial.set_timeout(Duration::from_secs(2)).unwrap();

        Self {
            current_color: Color::default(),
            serial,
        }
    }

    /// Performs initial setup with the serial connection to the Arduino, MUST
    /// be run before anything else
    pub fn setup(&mut self) {
        // Read the initial statement "I\r\n" that the Arduino sends
        let mut read_buf: [u8; 3] = [0; 3];
        self.serial
            .read_exact(&mut read_buf)
            .expect("Couldn't read initializer bytes");

        // Send the default color
        let write_bytes: [u8; 5] = <[u8; 5]>::from(&self.current_color);
        self.serial
            .write_all(&write_bytes)
            .expect("Couldn't write default");

        // Receive confirmation bytes "C\r\n"
        let mut read_buf: [u8; 3] = [0; 3];
        self.serial
            .read_exact(&mut read_buf)
            .expect("Couldn't read initial confirmation");
    }

    /// Updates the current color
    pub fn update(&mut self, color: Color) {
        self.current_color = color;
    }

    /// Sends a color to the Arduino and awaits a response
    pub fn send_color(&mut self) {
        // Send the color
        let write_bytes: [u8; 5] = <[u8; 5]>::from(&self.current_color);
        self.serial
            .write_all(&write_bytes)
            .expect("Couldn't write color bytes");

        // Receive confirmation bytes "C\r\n"
        let mut read_buf: [u8; 3] = [0; 3];
        self.serial
            .read_exact(&mut read_buf)
            .expect("Couldn't read confirmation");
    }
}
