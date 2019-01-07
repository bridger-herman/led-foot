use std::io::{Read, Write};
use std::sync::Mutex;
use std::time::Duration;

use serial::{SerialPort, SystemPort};

use crate::color::Color;

#[macro_export]
macro_rules! led_system {
    () => {
        crate::led_system::LED_SYSTEM.lock().unwrap()
    };
}

lazy_static! {
    pub static ref LED_SYSTEM: Mutex<LedSystem> = Mutex::new({
        let mut system = LedSystem::default();
        system.setup();
        system
    });
}

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

impl Default for LedSystem {
    fn default() -> Self {
        Self {
            current_color: Color::default(),
            serial: {
                let mut serial = serial::open("/dev//ttyACM0").unwrap();
                serial.set_timeout(Duration::from_secs(2)).unwrap();
                serial
            },
        }
    }
}
