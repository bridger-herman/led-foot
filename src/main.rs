#[macro_use]
extern crate nickel;
extern crate rustc_serialize;
extern crate serial;

pub mod color;
pub mod led_system;

use std::sync::{Arc, Mutex};

use nickel::status::StatusCode;
use nickel::{HttpRouter, Nickel, StaticFilesHandler};

use crate::color::Color;
use crate::led_system::LedSystem;

fn main() {
    let mut server = Nickel::new();
    let led_system = Arc::new(Mutex::new(LedSystem::new("/dev/ttyACM0")));
    led_system.try_lock().unwrap().setup();

    server.utilize(StaticFilesHandler::new("static"));

    server.post(
        "/api/set-rgbw-r=:red&g=:green&b=:blue&w=:white",
        middleware! {
            |request, mut response|
            let red = request.param("red").unwrap().parse::<u8>().unwrap();
            let green = request.param("green").unwrap().parse::<u8>().unwrap();
            let blue = request.param("blue").unwrap().parse::<u8>().unwrap();
            let white = request.param("white").unwrap().parse::<u8>().unwrap();

            let mut led_sys = led_system.try_lock().unwrap();
            led_sys.update(Color::new(red, green, blue, white));
            led_sys.send_color();

            response.set(StatusCode::Ok);
            format!("Setting color {} {} {} {}", red, green, blue, white)
        },
    );

    server.listen("0.0.0.0:8000").expect("Failed to serve");
}
