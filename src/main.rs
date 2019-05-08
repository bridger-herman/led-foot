#[macro_use]
extern crate log;
extern crate simple_logger;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate chrono;
extern crate png;
extern crate rustc_serialize;
extern crate serial;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate actix_web;

#[macro_use]
pub mod state;

pub mod color;
pub mod led_scheduler;
pub mod led_sequence;
pub mod led_system;

use std::{env, io};
use std::collections::HashMap;
use std::thread;

use actix_files as fs;
use actix_session::{CookieSession, Session};
use actix_web::http::{header, Method, StatusCode};
use actix_web::{
    error, guard, middleware, web, App, Error, HttpRequest, HttpResponse,
    HttpServer, Result,
};
// use bytes::Bytes;
// use futures::unsync::mpsc;
use futures::{future::ok, Future, Stream};

use crate::color::Color;
use crate::led_scheduler::LedAlarm;

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
            state.changed_from_ui = state.active;
        }
        led_system!().update_color(&color);
        led_system!().run_sequence();
        led_state!().changed_from_ui = false;

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
            state.changed_from_ui = state.active;
        }
        led_system!().update_sequence(&data["name"]);
        thread::spawn(move || {
            led_system!().run_sequence();
        });
        led_state!().changed_from_ui = false;

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

fn main() -> io::Result<()> {
    let log_level = ::std::env::args().filter(|item| item == "-v").count();
    let log_level = match log_level {
        1 => ::log::Level::Info,
        2 => ::log::Level::Debug,
        3 => ::log::Level::Trace,
        _ => ::log::Level::Warn,
    };
    println!("Starting LED server with verbosity {:?}", log_level);

    let sys = actix_rt::System::new("led-foot");

    HttpServer::new(|| {
        App::new()
            // enable logger
            .wrap(middleware::Logger::default())
            // cookie session middleware
            .wrap(CookieSession::signed(&[0; 32]).secure(false))
            // static files
            .service(
                fs::Files::new("/sequences", "sequences").show_files_listing(),
            )
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
            // simple index
            .service(web::resource("/").to(
                |_: HttpRequest| -> Result<HttpResponse> {
                    Ok(HttpResponse::build(StatusCode::OK)
                        .content_type("text/html; charset=utf-8")
                        .body(include_str!("../templates/index.html")))
                },
            ))
    })
    .bind("0.0.0.0:8000")?
    .start();

    thread::spawn(move || loop {
        led_schedule!().one_frame();

        thread::sleep(std::time::Duration::from_secs(1));
    });

    println!("Starting http server: 0.0.0.0:8000");
    sys.run()
}
