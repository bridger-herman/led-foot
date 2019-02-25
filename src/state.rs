//! Various useful macros for changing the LED state

use std::sync::Mutex;

use crate::led_scheduler::LedScheduler;
use crate::led_system::LedSystem;

lazy_static! {
    pub static ref LED_SYSTEM: Mutex<LedSystem> = Mutex::new({
        let mut system = LedSystem::default();
        system.setup();
        system
    });
    pub static ref STATE: Mutex<LedState> = Mutex::new(LedState::default());
    pub static ref SCHEDULE: Mutex<LedScheduler> =
        Mutex::new(LedScheduler::default());
}

#[macro_export]
macro_rules! led_system {
    () => {
        crate::state::LED_SYSTEM.lock().unwrap()
    };
}

#[macro_export]
macro_rules! led_state {
    () => {
        crate::state::STATE.try_lock().unwrap()
    };
}

#[macro_export]
macro_rules! led_schedule {
    () => {
        crate::state::SCHEDULE.try_lock().unwrap()
    };
}

#[derive(Default)]
pub struct LedState {
    pub changed_from_ui: bool,
    pub active: bool,
}
