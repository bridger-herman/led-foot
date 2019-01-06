#[macro_use]
extern crate nickel;
extern crate serial;

use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use nickel::status::StatusCode;
use nickel::{HttpRouter, Nickel, StaticFilesHandler};
use serial::SerialPort;

fn main() {
    let mut server = Nickel::new();

    let mut ser = serial::open("/dev/ttyACM0").unwrap();
    ser.set_timeout(Duration::from_secs(2)).unwrap();

    let serial_wrapper = Arc::new(Mutex::new(ser));

    let mut read_buf: [u8; 3] = [0; 3];
    serial_wrapper
        .try_lock()
        .unwrap()
        .read_exact(&mut read_buf)
        .expect("Couldn't read 1");

    let write_bytes: [u8; 5] = [0, 0, 0, 0, 0];
    serial_wrapper
        .try_lock()
        .unwrap()
        .write_all(&write_bytes)
        .expect("Couldn't write");

    let mut read_buf: [u8; 3] = [0; 3];
    serial_wrapper
        .try_lock()
        .unwrap()
        .read_exact(&mut read_buf)
        .expect("Couldn't read 2");

    server.utilize(StaticFilesHandler::new("static"));
    server.post("/api/set-rgbw-r=:red&g=:green&b=:blue&w=:white", middleware! { |request, mut response|
        let red = request.param("red").unwrap().parse::<u8>().unwrap();
        let green = request.param("green").unwrap().parse::<u8>().unwrap();
        let blue = request.param("blue").unwrap().parse::<u8>().unwrap();
        let white = request.param("white").unwrap().parse::<u8>().unwrap();

        let write_bytes: [u8; 5] = [0, red, green, blue, white];
        serial_wrapper.try_lock().unwrap().write_all(&write_bytes).expect("Couldn't write");

        let mut read_buf: [u8; 3] = [0; 3];
        serial_wrapper.try_lock().unwrap().read_exact(&mut read_buf).expect("Couldn't read 2");

        response.set(StatusCode::Ok);
        format!("Setting color {} {} {} {}", red, green, blue, white)
    });

    server.listen("0.0.0.0:8000").expect("Failed to serve");
}
