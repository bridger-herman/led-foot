use std::path::Path;
use std::thread::sleep;
use std::time::{Duration, Instant};

use crate::color::Color;
use crate::led_sequence::{LedSequence, RESOLUTION};

/// Controls the RGBW LEDs
pub struct LedSystem {
    pub current_color: Color,
    pub current_sequence: Option<LedSequence>,
}

impl LedSystem {
    /// Updates the current color
    pub fn update_color(&mut self, color: &Color) {
        self.current_sequence =
            Some(LedSequence::from_color_lerp(&self.current_color, &color));
    }

    /// Updates the current sequence directly
    pub fn update_sequence(&mut self, sequence_path: &str) {
        let seq = sequence_path.replace("png", "json");
        debug!("Sequence path: {:?}", seq);
        self.current_sequence = Some(LedSequence::from_color_points(
            &self.current_color,
            // TODO: Fix this on the javascript side (generate the colors from the
            // json)
            Path::new(&seq),
        ));
    }

    /// Runs through the current LED sequence
    pub fn run_sequence(&mut self) {
        led_state!().set_active(true);
        if let Some(ref mut seq) = self.current_sequence {
            let start = Instant::now();
            let mut previous_time = Instant::now();
            let mut current_time = Instant::now();
            let mut total_error = Duration::from_millis(0);
            for (i, color) in seq.enumerate() {
                let diff = current_time - previous_time;
                let delay = Duration::from_millis((1000.0 / RESOLUTION) as u64);
                let error = diff.checked_sub(delay).unwrap_or_default();
                total_error += error;
                if led_state!().changed_from_ui() {
                    info!("interrupting");
                    break;
                }
                let sleep_duration =
                    delay.checked_sub(total_error).unwrap_or_default();
                debug!(
                    "Sleeping for {:?} (total error {:?})",
                    sleep_duration, total_error
                );
                sleep(sleep_duration);
                self.current_color = color;

                debug!(
                    "Iteration {} - {}, {}, {}, {} ({:?})",
                    i,
                    self.current_color.r,
                    self.current_color.g,
                    self.current_color.b,
                    self.current_color.w,
                    delay,
                );

                serial_manager!().send_color(&self.current_color);
                subscribers!().send_color_update(&self.current_color);

                previous_time = current_time;
                current_time = Instant::now();
            }
            debug!("Time: {:?}", start.elapsed());
        }
        led_state!().set_active(false);
    }
}

impl Default for LedSystem {
    fn default() -> Self {
        Self {
            current_color: Color::default(),
            current_sequence: None,
        }
    }
}
