//! Manages the LED Arduino serial connection

use std::io::{Read, Write};
use std::time::Duration;

use serial::{SerialPort, SystemPort};

use crate::color::Color;

pub struct SerialManager {
    pub serial: Option<SystemPort>,
}

impl SerialManager {
    pub fn new(tty_name: &str) -> Self {
        let opened = serial::open(tty_name);
        let serial = if let Ok(mut ser) = opened {
            ser.set_timeout(Duration::from_secs(2)).unwrap();
            Some(ser)
        } else {
            error!(
                "Unable to initialize serial at {}. Using serial mockup",
                tty_name
            );
            None
        };

        Self { serial }
    }

    /// Performs initial setup with the serial connection to the Arduino, MUST
    /// be run before anything else
    pub fn setup(&mut self) {
        if let Some(ref mut ser) = self.serial {
            debug!("Setting up serial");
            // Read the initial statement "I\r\n" that the Arduino sends
            let mut read_buf: [u8; 3] = [0; 3];
            ser.read_exact(&mut read_buf)
                .expect("Couldn't read initializer bytes");

            // Send the default color
            let write_bytes: [u8; 6] =
                <[u8; 6]>::from(&led_system!().current_color);
            ser.write_all(&write_bytes).expect("Couldn't write default");

            // Receive confirmation bytes "C\r\n"
            let mut read_buf: [u8; 3] = [0; 3];
            ser.read_exact(&mut read_buf)
                .expect("Couldn't read initial confirmation");
            debug!("Received setup confirmation");
        }
    }

    /// Send a color to the Arduino over serial (or display the color on screen)
    pub fn send_color(&mut self, color: &Color) {
        if let Some(ref mut ser) = self.serial {
            // Send the color
            let write_bytes: [u8; 6] = <[u8; 6]>::from(color);
            debug!("sending bytes: {:?}", write_bytes);
            ser.write_all(&write_bytes)
                .expect("Couldn't write color bytes");

            // Receive confirmation bytes "C\r\n"
            let mut read_buf: [u8; 3] = [0; 3];
            ser.read_exact(&mut read_buf)
                .expect("Couldn't read confirmation");
        } else {
            println!(
                "\x1b[38;2;{};{};{}m{}\x1b[0m",
                color.r,
                color.g,
                color.b,
                "#".repeat(80),
            );
            println!(
                "\x1b[38;2;{};{};{}m{}\x1b[0m\n",
                color.w,
                color.w,
                color.w,
                "#".repeat(80),
            );
        }
    }
}

impl Default for SerialManager {
    fn default() -> Self {
        Self {
            serial: {
                let serial = serial::open("/dev/ttyACM0");
                if let Ok(mut ser) = serial {
                    ser.set_timeout(Duration::from_secs(2)).unwrap();
                    Some(ser)
                } else {
                    None
                }
            },
        }
    }
}

/// Convert to the format that the Arduino is expecting
impl From<&Color> for [u8; 6] {
    fn from(color: &Color) -> [u8; 6] {
        let color = color.clone().clamped();

        // Convert red and green to 16 bit integers (because they have the most
        // effect on luminance)
        let red_int = (color.r * f32::from(<u16>::max_value())).round() as u16;
        let red_byte1 = ((red_int & 0xff00) >> 8) as u8;
        let red_byte2 = (red_int & 0x00ff) as u8;

        let green_int =
            (color.g * f32::from(<u16>::max_value())).round() as u16;
        let green_byte1 = ((green_int & 0xff00) >> 8) as u8;
        let green_byte2 = (green_int & 0x00ff) as u8;

        let blue_byte = (color.b * f32::from(<u8>::max_value())).round() as u8;
        let white_byte = (color.w * f32::from(<u8>::max_value())).round() as u8;

        [
            red_byte1,
            red_byte2,
            green_byte1,
            green_byte2,
            blue_byte,
            white_byte,
        ]
    }
}
