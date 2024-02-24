use std::path::Path;
use std::thread::sleep;
use std::time::{Duration, Instant};
use std::iter::Iterator;

use crate::color::Color;
use crate::led_sequence::{LedSequence, RESOLUTION};
use crate::led_state::{LED_CONFIG, LED_STATE, SERIAL_MANAGER};

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

/// This impl is responsible for actually controlling the LEDs and room relays,
/// and nothing else. It runs on a separate thread, which loops and sends data
/// to the serial USB if there's a sequence running, otherwise it will spin.
impl LedSystem {
       pub fn new() -> Self {
        // Initialize the serial manager (needs to send/receive initialing message)

        let t = std::thread::spawn(|| LedSystem::led_sequence_worker());

        Self {
            sequence_thread: t,
        }
    }


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
        let mut status = LedSystemStatus {
            index: 0,
            start_time: Instant::now(),
            previous_time: Instant::now(),
            current_time: Instant::now(),
            total_error: Duration::from_millis(0),
        };
        loop {
            if let Ok(ref mut state) = LED_STATE.get().write() {
                if let Some(ref mut seq) = state.current_sequence.as_mut() {
                    if let Some(color) = seq.next() {
                        let diff = status.current_time - status.previous_time;
                        let delay = Duration::from_millis((1000.0 / RESOLUTION) as u64);
                        let error = diff.checked_sub(delay).unwrap_or_default();
                        status.total_error += error;

                        let sleep_duration =
                            delay.checked_sub(status.total_error).unwrap_or_default();
                        trace!(
                            "Sleeping for {:?} (total error {:?})",
                            sleep_duration,
                            status.total_error
                        );
                        sleep(sleep_duration);
                        state.current_color = color;

                        trace!(
                            "Iteration {} - {}, {}, {}, {} ({:?})",
                            status.index,
                            state.current_color.r,
                            state.current_color.g,
                            state.current_color.b,
                            state.current_color.w,
                            delay,
                        );

                        if let Ok(mut ser) = SERIAL_MANAGER.get().write() {
                            ser.send_color(&state.current_color);
                        }

                        status.previous_time = status.current_time;
                        status.current_time = Instant::now();
                        trace!("Time: {:?}", status.start_time.elapsed());
                    } else {

                    }

                }

                if state.shutdown {
                    break;
                }
            } else {
                break;
            }
            // TODO: this is spin-waiting and wasteful
            // std::thread::sleep(std::time::Duration::from_millis(1000));
        }
    } 
}




//     pub fn current_color(&self) -> &Color {
//         &self.current_color
//     }
//     pub fn current_sequence(&self) -> &Option<LedSequence> {
//         &self.current_sequence
//     }
//     pub fn active(&self) -> bool {
//         self.active
//     }
//     pub fn set_current_color(&mut self, color: &Color) {
//         self.current_color = color.clone();
//     }
//     pub fn set_current_sequence(&mut self, sequence: Option<LedSequence>) {
//         self.current_sequence = sequence;
//     }
//     pub fn set_active(&mut self, active: bool) {
//         self.active = active;
//     }

//     /// Updates the current color
//     pub fn update_color(&mut self, color: &Color) {
//         self.current_sequence =
//             Some(LedSequence::from_color_lerp(&self.current_color, &color));
//     }

//     /// Updates the current sequence directly
//     pub fn update_sequence(&mut self, sequence_path: &str) {
//         let seq = sequence_path.replace("png", "json");
//         debug!("Sequence path: {:?}", seq);
//         self.current_sequence = Some(LedSequence::from_color_points(
//             &self.current_color,
//             // TODO: Fix this on the javascript side (generate the colors from the
//             // json)
//             Path::new(&seq),
//         ));
//     }

//     /// Runs through the current LED sequence
//     pub fn run_sequence(&mut self) {
//         // Force there to not be any interrupt at the beginning
//         set_interrupt(false);

//         self.active = true;
//         if let Some(ref mut seq) = self.current_sequence {
//             let start = Instant::now();
//             let mut previous_time = Instant::now();
//             let mut current_time = Instant::now();
//             let mut total_error = Duration::from_millis(0);
//             for (i, color) in seq.enumerate() {
//                 let diff = current_time - previous_time;
//                 let delay = Duration::from_millis((1000.0 / RESOLUTION) as u64);
//                 let error = diff.checked_sub(delay).unwrap_or_default();
//                 total_error += error;

//                 // If the current sequence need to be interrupted because the user
//                 // or a schedule wants a different one
//                 if is_interrupted() {
//                     info!("Interrupting sequence");
//                     break;
//                 }
//                 let sleep_duration =
//                     delay.checked_sub(total_error).unwrap_or_default();
//                 trace!(
//                     "Sleeping for {:?} (total error {:?})",
//                     sleep_duration,
//                     total_error
//                 );
//                 sleep(sleep_duration);
//                 self.current_color = color;

//                 trace!(
//                     "Iteration {} - {}, {}, {}, {} ({:?})",
//                     i,
//                     self.current_color.r,
//                     self.current_color.g,
//                     self.current_color.b,
//                     self.current_color.w,
//                     delay,
//                 );

//                 if let Ok(mut ser) = SERIAL_MANAGER.get().write() {
//                     ser.send_color(&self.current_color);
//                 }
//                 // subscribers!().send_color_update(&self.current_color);

//                 previous_time = current_time;
//                 current_time = Instant::now();
//             }
//             trace!("Time: {:?}", start.elapsed());
//         }
//         self.active = false;
//     }
// }

// impl Default for LedSystem {
//     fn default() -> Self {
//         Self {
//             current_color: Color::default(),
//             current_sequence: None,
//             active: false,
//         }
//     }
// }
