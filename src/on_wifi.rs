//! Turn the LEDs on when specific devices connect to the network

use std::time::{Duration, Instant};

use net_sniffer::net_sniffer::NetSniffer;

use crate::color::Color;

pub struct WifiManager {
    sniffer: NetSniffer,

    /// Timeout when a device is connected (don't repeatedly ping)
    connected_timeout: Duration,

    /// Timeout when a device is disconnected (want to realize that it's
    /// reconnected as soon as possible)
    disconnected_timeout: Duration,

    last_triggered: Instant,
    device_connected: bool,
}

impl Default for WifiManager {
    fn default() -> Self {
        Self {
            sniffer: NetSniffer::default(),
            connected_timeout: Duration::from_millis(600_000),
            disconnected_timeout: Duration::from_millis(5_000),
            last_triggered: Instant::now(),
            device_connected: false,
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
            self.last_triggered = now;

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
