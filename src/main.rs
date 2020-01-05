#[macro_use]
extern crate log;
extern crate median;
extern crate simple_logger;
#[macro_use]
extern crate lazy_static;
extern crate chrono;
extern crate png;
extern crate serial;
#[macro_use]
extern crate serde_derive;
extern crate actix_web;
extern crate serde;
extern crate serde_json;

#[macro_use]
pub mod state;

pub mod color;
pub mod led_scheduler;
pub mod led_sequence;
pub mod led_system;
pub mod room_manager;
pub mod serial_manager;
pub mod subscribers;

use std::collections::HashMap;
use std::io;
use std::net::TcpListener;
use std::thread;
use std::time::Duration;

use actix_web::http::StatusCode;
use actix_web::{
    middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer, Result,
};
use futures::{Future, Stream};
use tungstenite::server::accept;

use crate::color::Color;
use crate::led_scheduler::LedAlarm;
use crate::room_manager::Room;

fn set_rgbw(
    payload: web::Payload,
) -> impl Future<Item = HttpResponse, Error = Error> {
    // Load the body
    payload.concat2().from_err().and_then(|body| {
        let color: Color =
            serde_json::from_str(std::str::from_utf8(&body).unwrap()).unwrap();

        info!("Setting color {:?}", color);
        {
            let mut state = led_state!();
            let active = state.active();
            state.set_changed_from_ui(active);
        }
        led_system!().update_color(&color);
        led_system!().run_sequence();
        led_state!().set_changed_from_ui(false);

        Ok(HttpResponse::Ok().json(color))
    })
}

fn set_sequence(
    payload: web::Payload,
) -> impl Future<Item = HttpResponse, Error = Error> {
    // Load the body
    payload.concat2().from_err().and_then(|body| {
        let data: HashMap<String, String> =
            serde_json::from_str(std::str::from_utf8(&body).unwrap()).unwrap();

        info!("Setting sequence {}", data["name"]);

        {
            let mut state = led_state!();
            let active = state.active();
            state.set_changed_from_ui(active);
        }
        led_system!().update_sequence(&data["name"]);
        thread::spawn(move || {
            led_system!().run_sequence();
        });
        led_state!().set_changed_from_ui(false);

        Ok(HttpResponse::Ok().json(data))
    })
}

fn set_schedule(
    payload: web::Payload,
) -> impl Future<Item = HttpResponse, Error = Error> {
    // Load the body
    payload.concat2().from_err().and_then(|body| {
        let data: Vec<LedAlarm> =
            serde_json::from_str(std::str::from_utf8(&body).unwrap()).unwrap();

        info!("Setting schedule");

        led_schedule!().reset_alarms(&data);
        led_schedule!().rewrite_schedule();

        Ok(HttpResponse::Ok().json(data))
    })
}

fn set_rooms(
    payload: web::Payload,
) -> impl Future<Item = HttpResponse, Error = Error> {
    // Load the body
    payload.concat2().from_err().and_then(|body| {
        let rooms: HashMap<Room, bool> =
            serde_json::from_str(std::str::from_utf8(&body).unwrap()).unwrap();

        rooms!().set_active_rooms(rooms.clone());

        Ok(HttpResponse::Ok().json(rooms))
    })
}

fn main() -> io::Result<()> {
    let log_level = ::std::env::args().filter(|item| item == "-v").count();
    let log_level = match log_level {
        1 => ::log::Level::Info,
        2 => ::log::Level::Debug,
        3 => ::log::Level::Trace,
        _ => ::log::Level::Warn,
    };
    println!("Starting LED server with verbosity {:?}", log_level);
    simple_logger::init_with_level(log_level).expect("Unable to initalize log");

    serial_manager!().setup();

    let sys = actix_rt::System::new("led-foot");

    HttpServer::new(|| {
        App::new()
            // enable logger
            .wrap(middleware::Logger::default())
            // static files
            .service(
                actix_files::Files::new(
                    "/led-foot-sequences",
                    "led-foot-sequences",
                )
                .show_files_listing(),
            )
            .service(actix_files::Files::new("/static", "static"))
            // api calls
            .service(web::resource("/api/get-schedule").to(
                |_: HttpRequest| -> Result<HttpResponse> {
                    Ok(HttpResponse::build(StatusCode::OK)
                        .content_type("application/json; charset=utf-8")
                        .body(
                            serde_json::to_string(&led_schedule!().alarms)
                                .expect("Failed to encode schedule"),
                        ))
                },
            ))
            .service(web::resource("/api/get-rgbw").to(
                |_: HttpRequest| -> Result<HttpResponse> {
                    Ok(HttpResponse::build(StatusCode::OK)
                        .content_type("application/json; charset=utf-8")
                        .body(
                            serde_json::to_string(&led_system!().current_color)
                                .expect("Failed to encode color"),
                        ))
                },
            ))
            .service(web::resource("/api/get-sequences").to(
                |_: HttpRequest| -> Result<HttpResponse> {
                    let dir_listing =
                        ::std::fs::read_dir("./led-foot-sequences").unwrap();
                    let sequences: Vec<String> = dir_listing
                        .map(|entry| {
                            entry.unwrap().path().to_str().unwrap().to_string()
                        })
                        .filter(|path_string| path_string.ends_with(".png"))
                        .collect();
                    Ok(HttpResponse::build(StatusCode::OK)
                        .content_type("application/json; charset=utf-8")
                        .body(
                            serde_json::to_string(&sequences)
                                .expect("Failed to encode sequences"),
                        ))
                },
            ))
            .service(web::resource("/api/get-rooms").to(
                |_: HttpRequest| -> Result<HttpResponse> {
                    Ok(HttpResponse::build(StatusCode::OK)
                        .content_type("application/json; charset=utf-8")
                        .body(
                            serde_json::to_string(&rooms!().active_rooms())
                                .expect("Failed to encode room list"),
                        ))
                },
            ))
            .service(
                web::resource("/api/set-rgbw")
                    .route(web::post().to_async(set_rgbw)),
            )
            .service(
                web::resource("/api/set-sequence")
                    .route(web::post().to_async(set_sequence)),
            )
            .service(
                web::resource("/api/set-schedule")
                    .route(web::post().to_async(set_schedule)),
            )
            .service(
                web::resource("/api/set-rooms")
                    .route(web::post().to_async(set_rooms)),
            )
            // simple index
            .service(web::resource("/").to(
                |_: HttpRequest| -> Result<HttpResponse> {
                    Ok(HttpResponse::build(StatusCode::OK)
                        .content_type("text/html; charset=utf-8")
                        .body(include_str!("../templates/index.html")))
                },
            ))
    })
    .workers(2)
    .bind("0.0.0.0:5000")?
    .start();

    // Start the websocket server
    let ws_server =
        TcpListener::bind("0.0.0.0:9001").expect("Unable to bind WS address");
    thread::spawn(move || {
        for stream in ws_server.incoming() {
            // Move the websocket to the subscriber list
            let ws = accept(stream.unwrap()).unwrap();
            subscribers!().add(ws);
        }
    });

    thread::spawn(move || loop {
        led_schedule!().one_frame();
        thread::sleep(Duration::from_secs(1));
    });

    println!("Starting http server: 0.0.0.0:5000");
    sys.run()
}
