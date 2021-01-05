use std::path::Path;
use std::thread::sleep;
use std::time::{Duration, Instant};

use crate::color::Color;
use crate::led_sequence::{LedSequence, RESOLUTION};
use crate::led_state::{is_interrupted, set_interrupt, SERIAL_MANAGER};

/// Controls the RGBW LEDs
pub struct LedSystem {
    /// Current color that the LEDs are
    current_color: Color,

    /// Current sequence the LEDs are running, if any
    current_sequence: Option<LedSequence>,

    /// Is the LED system currently running a sequence or transition?
    active: bool,
}

impl LedSystem {
    pub fn current_color(&self) -> &Color {
        &self.current_color
    }
    pub fn current_sequence(&self) -> &Option<LedSequence> {
        &self.current_sequence
    }
    pub fn active(&self) -> bool {
        self.active
    }
    pub fn set_current_color(&mut self, color: &Color) {
        self.current_color = color.clone();
    }
    pub fn set_current_sequence(&mut self, sequence: Option<LedSequence>) {
        self.current_sequence = sequence;
    }
    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }

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
        // Force there to not be any interrupt at the beginning
        set_interrupt(false);

        self.active = true;
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

                // If the current sequence need to be interrupted because the user
                // or a schedule wants a different one
                if is_interrupted() {
                    info!("Interrupting sequence");
                    break;
                }
                let sleep_duration =
                    delay.checked_sub(total_error).unwrap_or_default();
                trace!(
                    "Sleeping for {:?} (total error {:?})",
                    sleep_duration,
                    total_error
                );
                sleep(sleep_duration);
                self.current_color = color;

                trace!(
                    "Iteration {} - {}, {}, {}, {} ({:?})",
                    i,
                    self.current_color.r,
                    self.current_color.g,
                    self.current_color.b,
                    self.current_color.w,
                    delay,
                );

                if let Ok(mut ser) = SERIAL_MANAGER.get().write() {
                    ser.send_color(&self.current_color);
                }
                // subscribers!().send_color_update(&self.current_color);

                previous_time = current_time;
                current_time = Instant::now();
            }
            trace!("Time: {:?}", start.elapsed());
        }
        self.active = false;
    }
}

impl Default for LedSystem {
    fn default() -> Self {
        Self {
            current_color: Color::default(),
            current_sequence: None,
            active: false,
        }
    }
}
