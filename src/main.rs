#[macro_use]
extern crate log;
extern crate simple_logger;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate nickel;
extern crate png;
extern crate rustc_serialize;
extern crate serial;

#[macro_use]
pub mod state;

pub mod color;
pub mod led_sequence;
pub mod led_system;

use std::collections::HashMap;
use std::fs;

use nickel::mimes::MediaType;
use nickel::status::StatusCode;
use nickel::JsonBody;
use nickel::{HttpRouter, Nickel, StaticFilesHandler};
use rustc_serialize::json;

use crate::color::Color;

fn main() {
    simple_logger::init_with_level(::log::Level::Info).unwrap();
    let mut server = Nickel::new();

    server.utilize(StaticFilesHandler::new("static"));
    server.utilize(StaticFilesHandler::new("sequences"));

    // Render index.html with the current color values on the server
    server.get(
        "/",
        middleware! { |_, mut response|
            let mut template_data = HashMap::new();

            let current_color = &led_system!().current_color;
            template_data.insert("current_color", json::encode(&current_color).unwrap());
            let dir_listing = fs::read_dir("./sequences").unwrap();
            let sequences: Vec<String> = dir_listing.map(|entry| {
                entry.unwrap().path().file_name().unwrap().to_str().unwrap().to_string()
            }).collect();
            template_data.insert("sequences", json::encode(&sequences).unwrap());

            return response.render("templates/index.html", &template_data);
        },
    );

    // Long polling API call for changing the current color preview
    server.get(
        "/api/get-rgbw",
        middleware! { |_, mut response|
            let returned =
                json::encode(&led_system!().current_color.clone())
                    .expect("Failed to encode color");
            response.set(StatusCode::Ok);
            response.set(MediaType::Json);
            returned
        },
    );

    server.post(
        "/api/set-rgbw-r=:red&g=:green&b=:blue&w=:white",
        middleware! {
            |request, mut response|
            let red = request.param("red").unwrap().parse::<u8>().unwrap();
            let green = request.param("green").unwrap().parse::<u8>().unwrap();
            let blue = request.param("blue").unwrap().parse::<u8>().unwrap();
            let white = request.param("white").unwrap().parse::<u8>().unwrap();
            println!("Setting color {} {} {} {}", red, green, blue, white);

            led_state!().changed_from_ui = true;
            led_system!().update_color(&Color::new(red, green, blue, white));
            led_system!().run_sequence();

            response.set(StatusCode::Ok);
            format!("Setting color {} {} {} {}", red, green, blue, white)
        },
    );

    server.post(
        "/api/set-sequence",
        middleware! { |request, mut response|
            let data = request.json_as::<HashMap<String, String>>().unwrap();
            println!("Setting sequence {}", data["name"]);

            {
                let mut state = led_state!();
                state.changed_from_ui = state.active;
            }
            led_system!().update_sequence(&format!("./sequences/{}", data["name"]));
            led_system!().run_sequence();
            led_state!().changed_from_ui = false;

            response.set(StatusCode::Ok);
            format!("Setting sequence {}", data["name"])
        },
    );

    server.listen("0.0.0.0:8000").expect("Failed to serve");
}
