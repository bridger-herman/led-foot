//! Turn the LEDs on when specific devices connect to the network

use std::fs::File;
use std::io::Read;
use std::time::{Duration, Instant};

use chrono::{DateTime, Datelike, Local, TimeZone, NaiveDateTime};
use net_sniffer::net_sniffer::NetSniffer;
use sunrise;

use crate::color::Color;

const LAT_LONG_FILE: &str = "lat_long.txt";

pub struct WifiManager {
    sniffer: NetSniffer,

    /// Timeout when a device is connected (don't repeatedly ping)
    connected_timeout: Duration,

    /// Timeout when a device is disconnected (want to realize that it's
    /// reconnected as soon as possible)
    disconnected_timeout: Duration,

    last_triggered: Instant,
    device_connected: bool,

    lat_long: (f64, f64),
}

impl Default for WifiManager {
    fn default() -> Self {
        let mut file = File::open(LAT_LONG_FILE)
            .expect("Unable to open lat/long file");
        let mut buf = String::new();
        file.read_to_string(&mut buf).expect("Unable to read lat/long file");
        let lines: Vec<String> = buf.split('\n').map(String::from).collect();
        let lat_long = (
            lines[0].parse::<f64>().unwrap(),
            lines[1].parse::<f64>().unwrap(),
        );

        Self {
            sniffer: NetSniffer::default(),
            connected_timeout: Duration::from_millis(1_000),
            disconnected_timeout: Duration::from_millis(1_000),
            last_triggered: Instant::now(),
            device_connected: false,
            lat_long,
        }
    }
}

impl WifiManager {
    pub fn one_frame(&mut self) {
        let now = Instant::now();

        let timeout = if self.device_connected {
            self.connected_timeout
        } else {
            self.disconnected_timeout
        };

        if now - self.last_triggered > timeout {
            // let now = chrono::Local::now();
            let now = chrono::Local.ymd(2019, 4, 21).and_hms(1, 00, 00);

            let (sunrise, sunset) = sunrise::sunrise_sunset(
                self.lat_long.0,
                self.lat_long.1,
                now.year(),
                now.month(),
                now.day(),
            );

            let (sunrise, sunset) = (
                chrono::NaiveDateTime::from_timestamp(sunrise, 0),
                chrono::NaiveDateTime::from_timestamp(sunset, 0),
            );

            let sunrise_dt: chrono::DateTime<chrono::Local> =
                chrono::DateTime::from_utc(sunrise, now.offset().clone());
            let sunset_dt: chrono::DateTime<chrono::Local> =
                chrono::DateTime::from_utc(sunset, now.offset().clone());

            println!("sunrise {:?}", sunrise_dt);
            println!("sunset {:?}", sunset_dt);

            let sunrise_to_sunset = sunset_dt - sunrise_dt;
            let sunrise_to_now = now - sunrise_dt;
            println!("{:?}, {:?}", sunrise_to_now, sunrise_to_sunset);

            // Check if it's night or not
            if sunrise_to_now > sunrise_to_sunset {
                println!("It's night!");
            } else {
                println!("It's day!");
            }

            let connected = self.sniffer.compare_connections();

            if !connected.is_empty() && !self.device_connected {
                info!("New device connected: {:?}", connected);
                {
                    let mut state = led_state!();
                    state.changed_from_ui = state.active;
                }
                led_system!().update_color(&Color::new(255, 255, 255, 255));
                led_system!().run_sequence();
                led_state!().changed_from_ui = false;
            } else if !connected.is_empty() {
                info!("Device already connected: {:?}", connected);
            } else {
                info!("No devices connected");
            }

            self.device_connected = !connected.is_empty();
        }
    }
}
