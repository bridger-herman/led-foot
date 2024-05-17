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
    pub nominal_sleep_time: Duration,
    pub actual_sleep_time: Duration,
}

impl LedSystemStatus {
    pub fn new() -> Self {
        let sleep_time = Duration::from_millis((1000.0 / RESOLUTION) as u64);
        Self {
            index: 0,
            start_time: Instant::now(),
            current_time: Instant::now(),
            previous_time: Instant::now(),
            nominal_sleep_time: sleep_time,
            actual_sleep_time: sleep_time,
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
        let mut status = LedSystemStatus::new();
        let mut last_state = LedState::default();

        trace!("Set up LED System with temporal resolution {}fps, nominal sleep time per frame = {:?}", RESOLUTION, status.nominal_sleep_time);

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

                        // Calculate how long to sleep based on timing errors.
                        // Theorectially, we should be `index * delay` milleseconds along.
                        // But, with timing errors (serial delay) this isn't always the case.
                        // So, we correct for that here.
                        let nominal_current_time = status.nominal_sleep_time.checked_mul((status.index) as u32).unwrap_or_default();
                        let actual_current_time = status.start_time.elapsed();
                        let over_time = actual_current_time.checked_sub(nominal_current_time);
                        let under_time = nominal_current_time.checked_sub(actual_current_time);

                        trace!(
                            "Index {}, Nominal time {:?}, actual time {:?}, over time {:?}, under time {:?}",
                            status.index, nominal_current_time, actual_current_time, over_time, under_time
                        );

                        status.actual_sleep_time = if let Some(time_diff) = over_time {
                            status.actual_sleep_time.checked_sub(time_diff).expect(&format!("Cannot subtract negative over_time {:?} from sleep_time {:?}", time_diff, status.actual_sleep_time))
                        } else if let Some(time_diff) = under_time {
                            status.actual_sleep_time.checked_add(time_diff).expect(&format!("Cannot add negative over_time {:?} to sleep_time {:?}", time_diff, status.actual_sleep_time))
                        } else {
                            status.actual_sleep_time
                        };

                        // Update the timings (only do this when we're actively sending colors to serial)
                        status.previous_time = status.current_time;
                        status.current_time = Instant::now();
                        status.index += 1;
                    } else {
                        // hit the end of a sequence, or no sequence available
                        state.current_sequence = None;

                        // reset to beginning of whatever sequence is next
                        debug!("Stopped sequence, set index 0. Total time: {:?}", status.current_time - status.start_time);
                        status.reinitialize();

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

            trace!("Sleeping for: {:?}", status.actual_sleep_time);
            std::thread::sleep(status.actual_sleep_time);
            status.actual_sleep_time = status.nominal_sleep_time;
            trace!("Time: {:?}", status.start_time.elapsed());
        }
    }
}