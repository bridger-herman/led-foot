use std::io::{Read, Write};
use std::path::Path;
use std::thread::sleep;
use std::time::{Duration, Instant};

use serial::{SerialPort, SystemPort};

use crate::color::Color;
use crate::led_sequence::{LedSequence, RESOLUTION};

/// Controls the RGBW LEDs
pub struct LedSystem {
    pub current_color: Color,
    pub serial: Option<SystemPort>,
    pub current_sequence: Option<LedSequence>,
}

impl LedSystem {
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

        Self {
            current_color: Color::default(),
            serial,
            current_sequence: None,
        }
    }

    /// Performs initial setup with the serial connection to the Arduino, MUST
    /// be run before anything else
    pub fn setup(&mut self) {
        if let Some(ref mut ser) = self.serial {
            // Read the initial statement "I\r\n" that the Arduino sends
            let mut read_buf: [u8; 3] = [0; 3];
            ser.read_exact(&mut read_buf)
                .expect("Couldn't read initializer bytes");

            // Send the default color
            let write_bytes: [u8; 5] = <[u8; 5]>::from(&self.current_color);
            ser.write_all(&write_bytes).expect("Couldn't write default");

            // Receive confirmation bytes "C\r\n"
            let mut read_buf: [u8; 3] = [0; 3];
            ser.read_exact(&mut read_buf)
                .expect("Couldn't read initial confirmation");
        }
    }

    /// Updates the current color
    pub fn update_color(&mut self, color: &Color) {
        self.current_sequence =
            Some(LedSequence::from_color_lerp(&self.current_color, &color));
    }

    /// Updates the current sequence directly
    pub fn update_sequence(&mut self, sequence_path: &str) {
        self.current_sequence = Some(LedSequence::from_png(
            &self.current_color,
            Path::new(sequence_path),
        ));
    }

    /// Runs through the current LED sequence
    pub fn run_sequence(&mut self) {
        led_state!().active = true;
        if let Some(ref mut seq) = self.current_sequence {
            let start = Instant::now();
            let mut previous_time = Instant::now();
            let mut current_time = Instant::now();
            let mut total_error = Duration::from_millis(0);
            for (i, color) in seq.enumerate() {
                let diff = current_time - previous_time;
                let delay = Duration::from_millis((1000.0 / RESOLUTION) as u64);
                let error = diff.checked_sub(delay).unwrap_or_default();
                total_error += error;
                if led_state!().changed_from_ui {
                    info!("interrupting");
                    break;
                }
                let sleep_duration =
                    delay.checked_sub(total_error).unwrap_or_default();
                debug!("Sleeping for {:?}", sleep_duration);
                sleep(sleep_duration);
                self.current_color = color;

                debug!(
                    "{} - {}, {}, {}, {} ({:?})",
                    i,
                    self.current_color.r,
                    self.current_color.g,
                    self.current_color.b,
                    self.current_color.w,
                    delay,
                );
                debug!("{:?}, {:?}", diff, total_error);
                if let Some(ref mut ser) = self.serial {
                    // Send the color
                    let write_bytes: [u8; 5] =
                        <[u8; 5]>::from(&self.current_color);
                    ser.write_all(&write_bytes)
                        .expect("Couldn't write color bytes");

                    // Receive confirmation bytes "C\r\n"
                    let mut read_buf: [u8; 3] = [0; 3];
                    ser.read_exact(&mut read_buf)
                        .expect("Couldn't read confirmation");
                } else {
                    println!(
                        "\x1b[38;2;{};{};{}m{}\x1b[0m",
                        self.current_color.r,
                        self.current_color.g,
                        self.current_color.b,
                        "#".repeat(80),
                    );
                    println!(
                        "\x1b[38;2;{};{};{}m{}\x1b[0m\n",
                        self.current_color.w,
                        self.current_color.w,
                        self.current_color.w,
                        "#".repeat(80),
                    );
                }
                previous_time = current_time;
                current_time = Instant::now();
            }
            debug!("Time: {}", start.elapsed().as_secs());
        }
        led_state!().active = false;
    }
}

impl Default for LedSystem {
    fn default() -> Self {
        Self {
            current_color: Color::default(),
            serial: {
                let serial = serial::open("/dev/ttyACM0");
                if let Ok(mut ser) = serial {
                    ser.set_timeout(Duration::from_secs(2)).unwrap();
                    Some(ser)
                } else {
                    None
                }
            },
            current_sequence: None,
        }
    }
}
