//! Manages the LED Arduino serial connection

use std::io::{Error, ErrorKind, Read, Write};
use std::time::Duration;

use serial::{SerialPort, SystemPort};

use crate::color::Color;
use crate::room_manager::RoomManager;

// Magic numbers for color or room relay commands
const COLOR_CMD: u8 = 0xC0;
const ROOM_CMD: u8 = 0xF0;

// Magic numbers for each room
const LIVING_ROOM: u8 = 0x1A;
const OFFICE: u8 = 0x1C;
const BEDROOM: u8 = 0x18;

// Magic lengths
const UPDATE_BYTES: usize = 9;
const CONFIRMATION_BYTES: usize = 3;

pub struct SerialManager {
    pub serial: Option<SystemPort>,
}

impl SerialManager {
    pub fn new(tty_name: &str) -> Self {
        let opened = serial::open(tty_name);
        let serial = if let Ok(mut ser) = opened {
            ser.set_timeout(Duration::from_secs(2)).unwrap();
            warn!("Using serial: {}", tty_name);
            Some(ser)
        } else {
            warn!(
                "Unable to initialize serial at {}. Using Serial Mockup.",
                tty_name
            );
            None
        };

        let mut mgr = Self { serial };
        if let Err(io_err) = mgr.setup() {
            warn!(
                "Unable to initialize LEDs: {}. Using Serial Mockup.",
                io_err
            );
            Self { serial: None }
        } else {
            mgr
        }
    }

    /// Performs initial setup with the serial connection to the Arduino, MUST
    /// be run before anything else
    pub fn setup(&mut self) -> Result<(), Error> {
        if let Some(ref mut ser) = self.serial {
            debug!("Setting up serial");
            // Read the initial statement "I\r\n" that the Arduino sends
            let mut read_buf: [u8; CONFIRMATION_BYTES] =
                [0; CONFIRMATION_BYTES];
            ser.read_exact(&mut read_buf)?;

            if read_buf != "I\r\n".as_bytes() {
                return Err(Error::new(ErrorKind::Other, format!("Serial initialization reply didn't match `I` (received `{:?}` instead)", read_buf)));
            }

            // Send the default color to be black
            // let write_bytes: [u8; UPDATE_BYTES] =
            //     <[u8; UPDATE_BYTES]>::from(&led_system!().current_color);
            let write_bytes: [u8; UPDATE_BYTES] =
                [COLOR_CMD, 0, 0, 0, 0, 0, 0, 0, 0];
            ser.write_all(&write_bytes)?;

            // Receive confirmation bytes "C\r\n"
            let mut read_buf: [u8; CONFIRMATION_BYTES] =
                [0; CONFIRMATION_BYTES];
            ser.read_exact(&mut read_buf)?;

            if read_buf != "C\r\n".as_bytes() {
                return Err(Error::new(ErrorKind::Other, format!("Serial color setup reply didn't match `C` (received `{:?}` instead)", read_buf)));
            }

            debug!("Finished serial setup");
            Ok(())
        } else {
            // If there's no serial, there's no error to speak of
            Ok(())
        }
    }

    /// Send a color to the Arduino over serial (or display the color on screen)
    pub fn send_color(&mut self, color: &Color) {
        if let Some(ref mut ser) = self.serial {
            // Send the color
            let write_bytes: [u8; UPDATE_BYTES] =
                <[u8; UPDATE_BYTES]>::from(color);
            debug!("sending bytes: {:?}", write_bytes);
            ser.write_all(&write_bytes)
                .expect("Couldn't write color bytes");

            // Receive confirmation bytes "C\r\n"
            let mut read_buf: [u8; CONFIRMATION_BYTES] =
                [0; CONFIRMATION_BYTES];
            ser.read_exact(&mut read_buf)
                .expect("Couldn't read confirmation");

            if read_buf != "C\r\n".as_bytes() {
                error!(
                    "Serial reply didn't match `C` (received `{:?}` instead)",
                    read_buf
                );
            }
        } else {
            println!(
                "\x1b[38;2;{};{};{}m{}\x1b[0m",
                (color.r * f32::from(<u8>::max_value())) as u8,
                (color.g * f32::from(<u8>::max_value())) as u8,
                (color.b * f32::from(<u8>::max_value())) as u8,
                "#".repeat(80),
            );
            println!(
                "\x1b[38;2;{};{};{}m{}\x1b[0m\n",
                (color.w * f32::from(<u8>::max_value())) as u8,
                (color.w * f32::from(<u8>::max_value())) as u8,
                (color.w * f32::from(<u8>::max_value())) as u8,
                "#".repeat(80),
            );
        }
    }

    /// Send the current room state to the Arduino
    pub fn send_rooms(&mut self, state: &RoomManager) {
        if let Some(ref mut ser) = self.serial {
            // Send the color
            let write_bytes: [u8; UPDATE_BYTES] = rooms_to_bytes(state);
            trace!("sending bytes: {:?}", write_bytes);
            ser.write_all(&write_bytes)
                .expect("Couldn't write color bytes");

            // Receive confirmation bytes "R\r\n"
            let mut read_buf: [u8; CONFIRMATION_BYTES] =
                [0; CONFIRMATION_BYTES];
            ser.read_exact(&mut read_buf)
                .expect("Couldn't read confirmation");

            if read_buf != "R\r\n".as_bytes() {
                error!(
                    "Serial reply didn't match `R` (received `{:?}` instead)",
                    read_buf
                );
            }
        } else {
            println!("Serial Mockup: {:?}", state);
        }
    }
}

impl Default for SerialManager {
    fn default() -> Self {
        Self::new("/dev/ttyACM0")
    }
}

/// Convert to the format that the Arduino is expecting, including the prefix
/// magic number COLOR_CMD
impl From<&Color> for [u8; UPDATE_BYTES] {
    fn from(color: &Color) -> [u8; UPDATE_BYTES] {
        let color = color.clamped();

        let red_int = (color.r * f32::from(<u16>::max_value())).round() as u16;
        let red_byte1 = ((red_int & 0xff00) >> 8) as u8;
        let red_byte2 = (red_int & 0x00ff) as u8;

        let green_int =
            (color.g * f32::from(<u16>::max_value())).round() as u16;
        let green_byte1 = ((green_int & 0xff00) >> 8) as u8;
        let green_byte2 = (green_int & 0x00ff) as u8;

        let blue_int = (color.b * f32::from(<u16>::max_value())).round() as u16;
        let blue_byte1 = ((blue_int & 0xff00) >> 8) as u8;
        let blue_byte2 = (blue_int & 0x00ff) as u8;

        let white_int =
            (color.w * f32::from(<u16>::max_value())).round() as u16;
        let white_byte1 = ((white_int & 0xff00) >> 8) as u8;
        let white_byte2 = (white_int & 0x00ff) as u8;

        [
            COLOR_CMD,
            red_byte1,
            red_byte2,
            green_byte1,
            green_byte2,
            blue_byte1,
            blue_byte2,
            white_byte1,
            white_byte2,
        ]
    }
}

/// Convert to the format that the Arduino is expecting, including the prefix
/// magic number ROOM_CMD
fn rooms_to_bytes(rooms: &RoomManager) -> [u8; UPDATE_BYTES] {
    let living = if let Some(room_val) = rooms.living_room { if room_val {LIVING_ROOM} else { 0x00 } } else { 0x00 };
    let office = if let Some(room_val) = rooms.office { if room_val {OFFICE} else { 0x00 } } else { 0x00 };
    let bedroom = if let Some(room_val) = rooms.bedroom { if room_val {BEDROOM} else { 0x00 } } else { 0x00 };
    [
        ROOM_CMD,
        living,
        office,
        bedroom,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
    ]
}
