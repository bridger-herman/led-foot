#[macro_use]
extern crate log;

pub mod color;
pub mod led_scheduler;
pub mod led_sequence;
pub mod led_state;
pub mod led_system;
pub mod room_manager;
pub mod serial_manager;
pub mod wemo_manager;
// pub mod subscribers;

use std::collections::HashMap;
use std::thread;
use std::time::Duration;

use actix_files::Files;
use actix_web::error::ErrorInternalServerError;
use actix_web::{
    get, middleware, post, web, App, Error, HttpResponse, HttpServer, Result,
};

use crate::color::Color;
use crate::led_scheduler::LedAlarm;
use crate::led_state::{
    set_interrupt, LED_SCHEDULER, LED_SYSTEM, ROOM_MANAGER, WEMO_MANAGER,
};
use crate::room_manager::RoomManager;

#[post("/api/wemo")]
async fn wemo(
    payload: web::Json<HashMap<String, String>>,
) -> Result<HttpResponse, Error> {
    for (wemo, cmd) in payload.iter() {
        WEMO_MANAGER.get().send_wemo_command(wemo, cmd);
    }
    Ok(HttpResponse::Ok().json("{}"))
}

#[get("/api/get-rgbw")]
async fn get_rgbw() -> Result<HttpResponse, Error> {
    if let Ok(ref sys) = LED_SYSTEM.get().read() {
        Ok(HttpResponse::Ok().json(&sys.current_color()))
    } else {
        Err(ErrorInternalServerError("Unable to get RGBW data"))
    }
}

#[post("/api/set-rgbw")]
async fn set_rgbw(payload: web::Json<Color>) -> Result<HttpResponse, Error> {
    // Signal that we need to interrupt the current sequence
    set_interrupt(true);

    // Then, spawn a thread to handle the actual LED code
    std::thread::spawn(move || {
        if let Ok(mut sys) = LED_SYSTEM.get().write() {
            sys.update_color(&payload);
            sys.run_sequence();
        } else {
            error!("Unable to acquire lock on LED system");
        };
    });
    Ok(HttpResponse::Ok().json("{}"))
}

#[get("/api/get-rooms")]
async fn get_rooms() -> Result<HttpResponse, Error> {
    if let Ok(ref mgr) = ROOM_MANAGER.get().read() {
        Ok(HttpResponse::Ok().json(mgr.active_rooms()))
    } else {
        Err(ErrorInternalServerError("Unable to get room data"))
    }
}

#[post("/api/set-rooms")]
async fn set_rooms(
    payload: web::Json<RoomManager>,
) -> Result<HttpResponse, Error> {
    if let Ok(mut mgr) = ROOM_MANAGER.get().write() {
        mgr.set_active_rooms(&payload);
        Ok(HttpResponse::Ok().json(mgr.active_rooms()))
    } else {
        Err(ErrorInternalServerError("Unable to set room data"))
    }
}

#[get("/api/get-sequences")]
async fn get_sequences() -> Result<HttpResponse, Error> {
    let dir_listing = ::std::fs::read_dir("./led-foot-sequences")?;
    let mut sequences: Vec<String> = dir_listing
        .map(|entry| entry.unwrap().path().to_str().unwrap().to_string())
        .filter(|path_string| path_string.ends_with(".png"))
        .collect();
    sequences.sort();

    Ok(HttpResponse::Ok()
        .content_type("application/json; charset=utf-8")
        .body(
            serde_json::to_string(&sequences)
                .expect("Failed to encode sequence list"),
        ))
}

#[post("/api/set-sequence")]
async fn set_sequence(
    payload: web::Json<HashMap<String, String>>,
) -> Result<HttpResponse, Error> {
    let sequence_name = payload["name"].clone();
    debug!("Setting sequence {}", sequence_name);
    // Signal that we need to interrupt the current sequence
    set_interrupt(true);

    // Then, spawn a thread to handle the actual LED code
    std::thread::spawn(move || {
        if let Ok(mut sys) = LED_SYSTEM.get().write() {
            sys.update_sequence(&sequence_name);
            sys.run_sequence();
        } else {
            error!("Unable to acquire lock on LED system");
        };
    });
    Ok(HttpResponse::Ok().json(format!("{{\"name\": {}}}", payload["name"])))
}

#[get("/api/get-schedule")]
async fn get_schedule() -> Result<HttpResponse, Error> {
    if let Ok(sched) = LED_SCHEDULER.get().read() {
        Ok(HttpResponse::Ok().json(&sched.alarms))
    } else {
        Err(ErrorInternalServerError(
            "Unable to obtain lock on scheduler",
        ))
    }
}

#[post("/api/set-schedule")]
async fn set_schedule(
    payload: web::Json<Vec<LedAlarm>>,
) -> Result<HttpResponse, Error> {
    info!("Setting schedule");

    if let Ok(mut sched) = LED_SCHEDULER.get().write() {
        sched.reset_alarms(&payload);
        sched.rewrite_schedule();
        Ok(HttpResponse::Ok().json("{}"))
    } else {
        Err(ErrorInternalServerError(
            "Unable to obtain lock on scheduler",
        ))
    }
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

    // Initialize state
    led_state::init();

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
            // The rest of the services for controlling the LEDs
            .service(get_rgbw)
            .service(set_rgbw)
            .service(get_rooms)
            .service(set_rooms)
            .service(get_sequences)
            .service(set_sequence)
            .service(get_schedule)
            .service(set_schedule)
            .service(wemo)
    })
    .workers(4)
    .bind("0.0.0.0:5000")?;

    thread::spawn(move || loop {
        if let Ok(mut sched) = LED_SCHEDULER.get().write() {
            sched.one_frame();
        }
        thread::sleep(Duration::from_secs(1));
    });

    server.run().await
}
