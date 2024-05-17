#[macro_use]
extern crate log;

pub mod color;
pub mod led_config;
pub mod led_sequence;
pub mod led_state;
pub mod led_system;
pub mod rooms;
pub mod serial_manager;

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

// API Endpoints:
// /api/get-rgbw
// /api/set-rgbw
//
// /api/get-sequence
// /api/set-sequence
//
// /api/get-rooms
// /api/set-rooms


/// Retrieve the current color that the LEDs are on
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

/// Retrieve the color that the LEDs WILL on when a transition-in-progress is complete
async fn get_color_future() -> HttpResponse {
    if let Ok(led_state) = LED_STATE.get().read() {
        // Default to the current color and hope for the best
        let mut send_color = led_state.current_color.clone();

        // If it's a transition / one-off sequence, use the last element instead
        if let Some(ref seq) = led_state.current_sequence {
            if !seq.info.repeat {
                if let Some(col) = seq.colors.back() {
                    send_color = col.clone();
                }
            }
        }

        HttpResponse::Ok()
            .content_type(ContentType::json())
            .json(send_color)
    } else {
        error!("Error on /api/get-color: can't get lock on state");
        HttpResponse::InternalServerError().into()
    }
}

/// Set the RGBW color for the LEDs and automatically begin a sequence w/transition
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

/// Get the sequence that is currently running
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

/// Switch to a new sequence
async fn set_sequence(payload: String) -> HttpResponse {
    if let Ok(ref mut led_state) = LED_STATE.get().write() {
        // TODO: This is hacky - check if the payload includes the magic phrase
        let seq_with_transition = if payload.starts_with("fade-to-black-") {
            let tokens = payload.split("-");
            debug!("Parsed fade to black tokens {:?}", tokens);
            let duration_result= tokens
                .last()
                .expect("Unable to get final token in payload string for /api/set-sequence")
                .parse::<f32>();
            if let Ok(duration) = duration_result {
                Ok(LedSequence::fade_to_black(&led_state.current_color, duration))
            } else {
                Ok(LedSequence::fade_to_black(&led_state.current_color, crate::led_sequence::FADE_DURATION))
            }
        } else {
            let seq_path = payload.replace("png", "json");
            debug!("Sequence path: {:?}", seq_path);
            LedSequence::from_color_points(
                &led_state.current_color,
                // TODO: Fix this on the javascript side (generate the colors from the
                // json)
                Path::new(&seq_path),
            )
        };

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


async fn list_sequences() -> HttpResponse {
    if let Ok(paths) = std::fs::read_dir(led_sequence::SEQUENCE_PATH) {

        let sequences_list = paths
            .filter_map(|e| e.ok())
            .map(|p| p.path().to_string_lossy().into_owned())
            .filter(|p| p.ends_with(".png"))
            .collect::<Vec<_>>();

        HttpResponse::Ok()
            .content_type(ContentType::plaintext())
            .body(sequences_list.join("\n"))
    } else {
        error!("Error on /api/list-sequences: can't read directory {}", led_sequence::SEQUENCE_PATH);
        HttpResponse::InternalServerError().into()
    }
}


/// Get the rooms that are currently enabled
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

/// Set the rooms that are currently enabled
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

/// Unused base API URL for LED Foot
async fn base_api() -> HttpResponse {
    HttpResponse::Ok()
        .content_type(ContentType::plaintext())
        .body("API for Led Foot")
}

/// index.html
#[get("/")]
async fn index() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../index.html"))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    ::env_logger::init();
    println!("Starting LED server with log level {:?}", ::log::max_level());

    let server = HttpServer::new(|| {
        App::new()
            // Enable the logger.
            .wrap(middleware::Logger::default())
            // Serve sequences as static files (allow to see file list if user wants)
            .service(
                Files::new("/led-foot-sequences", led_sequence::SEQUENCE_PATH)
                    .show_files_listing(),
            )
            // Serve the rest of the static files
            .service(Files::new("/static", "static"))
            // index.html
            .service(index)
            // The rest of the routes for controlling the LEDs
            .route("/api", web::get().to(base_api))
            .route("/api/get-color", web::get().to(get_color))
            .route("/api/get-color-future", web::get().to(get_color_future))
            .route("/api/set-color", web::post().to(set_color))
            .route("/api/get-sequence", web::get().to(get_sequence))
            .route("/api/set-sequence", web::post().to(set_sequence))
            .route("/api/list-sequences", web::get().to(list_sequences))
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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serial_connection() {
        let mut mgr = serial_manager::SerialManager::new("/dev/ttyACM0");
        mgr.send_rooms(&Rooms { living_room: true, office: true, bedroom: true });
        let color_to_send: Color = Color::new(1.0, 1.0, 1.0, 1.0);

        const N_TESTS: usize = 10000;
        for n in 0..N_TESTS {
            let last = std::time::Instant::now();
            mgr.send_color(&color_to_send);
            let next = std::time::Instant::now();
            println!("iteration {}, elapsed {:?}", n, next - last);
        }
    }
}
