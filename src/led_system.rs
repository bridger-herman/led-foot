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
        self.current_sequence = Some(LedSequence::from_png(
            &self.current_color,
            Path::new(sequence_path),
        ));
    }

    /// Runs through the current LED sequence
    pub fn run_sequence(&mut self) {
        led_state!().active = true;
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
                if led_state!().changed_from_ui {
                    info!("interrupting");
                    break;
                }
                let sleep_duration =
                    delay.checked_sub(total_error).unwrap_or_default();
                debug!("Sleeping for {:?}", sleep_duration);
                sleep(sleep_duration);
                self.current_color = color;

                debug!(
                    "{} - {}, {}, {}, {} ({:?})",
                    i,
                    self.current_color.r,
                    self.current_color.g,
                    self.current_color.b,
                    self.current_color.w,
                    delay,
                );
                debug!("{:?}, {:?}", diff, total_error);

                serial_manager!().send_color(&self.current_color);

                previous_time = current_time;
                current_time = Instant::now();
            }
            debug!("Time: {}", start.elapsed().as_secs());
        }
        led_state!().active = false;
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
