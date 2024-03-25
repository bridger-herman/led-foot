use std::iter::Iterator;
use std::time::{Duration, Instant};

use crate::led_sequence::RESOLUTION;
use crate::led_state::{LedState, LED_STATE, SERIAL_MANAGER};

/// Controls the RGBW LEDs.
pub struct LedSystem {
    sequence_thread: std::thread::JoinHandle<()>,
}

struct LedSystemStatus {
    pub index: usize,
    pub start_time: Instant,
    pub previous_time: Instant,
    pub current_time: Instant,
    pub total_error: Duration,
}

impl LedSystemStatus {
    pub fn new() -> Self {
        Self {
            index: 0,
            start_time: Instant::now(),
            current_time: Instant::now(),
            previous_time: Instant::now(),
            total_error: Duration::from_millis(0),
        }
    }

    pub fn reinitialize(&mut self) {
        *self = Self::new();
    }
}

/// This impl is responsible for actually controlling the LEDs and room relays,
/// and nothing else. It runs on a separate thread, which loops and sends data
/// to the serial USB if there's a sequence running, otherwise it will spin.
impl LedSystem {
    /// Create a new LedSystem instance. Should be a ~singleton.
    pub fn new() -> Self {
        let t = std::thread::spawn(|| LedSystem::led_sequence_worker());

        Self { sequence_thread: t }
    }

    /// Shut down this LedSystem instance and wait for the sequence worker thread to join.
    pub fn shutdown(self) -> Result<(), &'static str> {
        debug!("Shutting down LED system...");
        if let Ok(mut state) = LED_STATE.get().write() {
            state.shutdown = true;
        } else {
            return Err("Unable to obtain lock on state");
        }
        self.sequence_thread
            .join()
            .map_err(|_| "Unable to shutdown LED system worker thread")
    }

    fn led_sequence_worker() {
        let delay = Duration::from_millis((1000.0 / RESOLUTION) as u64);
        let mut status = LedSystemStatus::new();
        let mut last_state = LedState::default();

        loop {
            if let Ok(ref mut state) = LED_STATE.get().write() {
                // Update the rooms (if changed)
                if last_state.current_rooms != state.current_rooms {
                    if let Ok(mut ser) = SERIAL_MANAGER.get().write() {
                        ser.send_rooms(&state.current_rooms);
                    }
                }

                // Update the sequence & current color, if it exists
                if let Some(ref mut seq) = state.current_sequence.as_mut() {
                    if let Some(color) = seq.next() {
                        // starting a new sequence
                        if status.index == 0 {
                            status.reinitialize();
                        }

                        trace!(
                            "Iteration {} - {}, {}, {}, {}",
                            status.index,
                            state.current_color.r,
                            state.current_color.g,
                            state.current_color.b,
                            state.current_color.w,
                        );

                        // Send color to serial
                        if let Ok(mut ser) = SERIAL_MANAGER.get().write() {
                            ser.send_color(&color);
                        }

                        // Update color in state
                        state.current_color = color;

                        // Update timing errors
                        let diff = status.current_time - status.previous_time;
                        let error = diff.checked_sub(delay).unwrap_or_default();
                        status.total_error += error;

                        // Update the timings (only do this when we're actively sending colors to serial)
                        status.previous_time = status.current_time;
                        status.current_time = Instant::now();
                        status.index += 1;
                    } else {
                        // hit the end of a sequence, or no sequence available
                        state.current_sequence = None;

                        // reset to beginning of whatever sequence is next
                        status.reinitialize();
                        debug!("Stopped sequence, set index 0");

                        // TODO: use LED_ACTIVE here to avoid spin-waiting
                    }
                }

                if state.shutdown {
                    debug!("Shutting down / exiting LED spin");
                    break;
                }

                last_state = state.clone();
            } else {
                break;
            }

            let sleep_duration =
                delay.checked_sub(status.total_error).unwrap_or_default();
            trace!(
                "Sleeping for {:?} (total error {:?})",
                sleep_duration, status.total_error
            );
            std::thread::sleep(sleep_duration);
            trace!("Time: {:?}", status.start_time.elapsed());
        }
    }
}