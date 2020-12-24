#[macro_use]
extern crate log;

pub mod led_state;
pub mod color;
// pub mod led_scheduler;
// pub mod led_sequence;
// pub mod led_system;
pub mod room_manager;
pub mod serial_manager;
// pub mod subscribers;

use actix_files::Files;
use actix_web::{web, get, post, middleware, App, Error, HttpServer, HttpRequest,
HttpResponse, Result};
use actix_web::error::ErrorInternalServerError;
use actix_web::http::{header, StatusCode};

// use crate::color::Color;
// use crate::led_scheduler::LedAlarm;
use crate::room_manager::RoomManager;
// use crate::led_state::LedState;
// use crate::room_manager::RoomManager;
use crate::led_state::ROOM_MANAGER;



#[get("/api/get-rgbw")]
async fn get_rgbw() -> Result<HttpResponse> {
    Ok(HttpResponse::build(StatusCode::OK)
        .content_type("application/json; charset=utf-8")
        .body("{\"test\": 2}")
    )
}

#[post("/api/set-rgbw")]
async fn set_rgbw(req: HttpRequest) -> Result<HttpResponse> {
    println!("{:?}", req);
    Ok(HttpResponse::build(StatusCode::OK)
        .content_type("application/json; charset=utf-8")
        .body("{\"test\": 2}")
    )
}

#[get("/api/get-rooms")]
async fn get_rooms() -> Result<HttpResponse, Error> {
    if let Ok(ref mgr) = ROOM_MANAGER.get().read() {
        Ok(HttpResponse::Ok().json(
            serde_json::to_string(mgr.active_rooms()).expect("Failed to encode room list")
        ))
    } else {
        Err(ErrorInternalServerError("Unable to get room data"))
    }
}

#[post("/api/set-rooms")]
async fn set_rooms(payload: web::Json<RoomManager>) -> Result<HttpResponse> {
    if let Ok(mut mgr) = ROOM_MANAGER.get().write() {
        mgr.set_active_rooms(&payload);
        Ok(HttpResponse::Ok().json("{}"))
    } else {
        Err(ErrorInternalServerError("Unable to set room data"))
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    let log_level = ::std::env::args().filter(|item| item == "-v").count();
    let log_level = match log_level {
        1 => ::log::LevelFilter::Info,
        2 => ::log::LevelFilter::Debug,
        3 => ::log::LevelFilter::Trace,
        _ => ::log::LevelFilter::Warn,
    };
    println!("Starting LED server with verbosity {:?}", log_level);
    ::simple_logger::SimpleLogger::new()
        .with_level(log_level)
        .init()
        .expect("Unable to initialize log");

    // Initialize state
    led_state::init();

    HttpServer::new(|| {
        App::new()
            // Enable the logger.
            .wrap(middleware::Logger::default())
            // Serve sequences as static files (allow to see file list if user wants)
            .service(Files::new("/led-foot-sequences", "led-foot-sequences").show_files_listing())
            // Serve the rest of the static files
            .service(Files::new("/static", "static").index_file("index.html"))
            // Redirect to index
            .service(web::resource("/").route(web::get().to(|| {
                HttpResponse::Found()
                    .header(header::LOCATION, "static/index.html")
                    .finish()
            })))

            // The rest of the services for controlling the LEDs
            .service(get_rgbw)
            .service(set_rgbw)
            .service(get_rooms)
            .service(set_rooms)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}