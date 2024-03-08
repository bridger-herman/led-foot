#[macro_use]
extern crate log;

pub mod color;
pub mod led_config;
pub mod led_scheduler;
pub mod led_sequence;
pub mod led_state;
pub mod led_system;
pub mod rooms;
pub mod serial_manager;
pub mod wemo_manager;

use std::path::Path;

use actix_files::Files;
use actix_web::http::header::ContentType;
use actix_web::{
    get, middleware, web, App, HttpResponse, HttpServer,
};

use crate::color::Color;
use crate::led_sequence::LedSequence;
use crate::led_state::LED_STATE;
use crate::rooms::Rooms;

// #[post("/api/wemo")]
// async fn wemo(
//     payload: web::Json<HashMap<String, String>>,
// ) -> Result<HttpResponse, Error> {
//     for (wemo, cmd) in payload.iter() {
//         WEMO_MANAGER.get().send_wemo_command(wemo, cmd);
//     }
//     Ok(HttpResponse::Ok().json("{}"))
// }

// API Endpoints:
// /api/get-rgbw
// /api/set-rgbw
//
// /api/get-sequence
// /api/set-sequence
//
// /api/get-rooms
// /api/set-rooms


async fn get_color() -> HttpResponse {
    if let Ok(led_state) = LED_STATE.get().read() {
        HttpResponse::Ok()
            .content_type(ContentType::json())
            .json(led_state.current_color.clone())
    } else {
        error!("Error on /api/get-color: can't get lock on state");
        HttpResponse::InternalServerError().into()
    }
}

async fn set_color(payload: web::Json<Color>) -> HttpResponse {
    debug!("Color: {:?}", payload);
    if let Ok(mut led_state) = LED_STATE.get().write() {
        // does not directly set color - smoothly interpolates to the color.
        let seq_with_transition = LedSequence::from_color_lerp(
            &led_state.current_color,
            &payload,
        );
        led_state.current_sequence = Some(seq_with_transition);

        HttpResponse::Ok()
            .content_type(ContentType::plaintext())
            .body(format!("Set color to {:?}", payload))
    } else {
        error!("Error on /api/set-color: can't get lock on state");
        HttpResponse::InternalServerError().into()
    }
}

async fn get_sequence() -> HttpResponse {
    if let Ok(led_state) = LED_STATE.get().read() {
        let current_sequence_name = led_state.current_sequence.clone().map_or(None, |s| Some(s.info.name));
        HttpResponse::Ok()
            .content_type(ContentType::json())
            .json(current_sequence_name)
    } else {
        error!("Error on /api/get-sequence: can't get lock on state");
        HttpResponse::InternalServerError().into()
    }
}

async fn set_sequence(payload: String) -> HttpResponse {
    if let Ok(ref mut led_state) = LED_STATE.get().write() {
        let seq_path = payload.replace("png", "json");
        debug!("Sequence path: {:?}", seq_path);
        let seq_with_transition = LedSequence::from_color_points(
            &led_state.current_color,
            // TODO: Fix this on the javascript side (generate the colors from the
            // json)
            Path::new(&seq_path),
        );

        if let Err(e) = seq_with_transition {
            return HttpResponse::BadRequest()
                .content_type(ContentType::plaintext())
                .body(format!("No sequence named {:?}; {:?}", payload, e))
        }

        led_state.current_sequence = seq_with_transition.ok();

        HttpResponse::Ok()
            .content_type(ContentType::plaintext())
            .body(format!("Set sequence to to {:?}", payload))
    } else {
        error!("Error on /api/set-sequence: can't get lock on state");
        HttpResponse::InternalServerError().into()
    }
}

async fn get_rooms() -> HttpResponse {
    if let Ok(led_state) = LED_STATE.get().read() {
        HttpResponse::Ok()
            .content_type(ContentType::json())
            .json(led_state.current_rooms.clone())
    } else {
        error!("Error on /api/get-rooms: can't get lock on state");
        HttpResponse::InternalServerError().into()
    }
}

async fn set_rooms(payload: web::Json<Rooms>) -> HttpResponse {
    if let Ok(mut led_state) = LED_STATE.get().write() {
        led_state.current_rooms = payload.clone();
        HttpResponse::Ok()
            .content_type(ContentType::plaintext())
            .body(format!("Set rooms to {:?}", payload))
    } else {
        error!("Error on /api/set-rooms: can't get lock on state");
        HttpResponse::InternalServerError().into()
    }
}

async fn base_api() -> HttpResponse {
    HttpResponse::Ok()
        .content_type(ContentType::plaintext())
        .body("API for Led Foot")
}

#[get("/")]
async fn index() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../index.html"))
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

    let server = HttpServer::new(|| {
        App::new()
            // Enable the logger.
            .wrap(middleware::Logger::default())
            // Serve sequences as static files (allow to see file list if user wants)
            .service(
                Files::new("/led-foot-sequences", "led-foot-sequences")
                    .show_files_listing(),
            )
            // Serve the rest of the static files
            .service(Files::new("/static", "static"))
            // index.html
            .service(index)
            // The rest of the routes for controlling the LEDs
            .route("/api", web::get().to(base_api))
            .route("/api/get-color", web::get().to(get_color))
            .route("/api/set-color", web::post().to(set_color))
            .route("/api/get-sequence", web::get().to(get_sequence))
            .route("/api/set-sequence", web::post().to(set_sequence))
            .route("/api/get-rooms", web::get().to(get_rooms))
            .route("/api/set-rooms", web::post().to(set_rooms))
    })
    .bind("0.0.0.0:5000")?;

    // Initialize state
    led_state::init_global_state();

    // Start the LED System
    let sys = led_system::LedSystem::new();

    server.run()
        .await
        .and_then(|_| {
            sys.shutdown()
            .map(|r| {
                debug!("LED system shutdown normally");
                r
            })
            .map_err(|msg| std::io::Error::new(std::io::ErrorKind::Other, msg))
        })
}
