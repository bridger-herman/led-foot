//! Various useful macros for changing the LED state

use std::sync::Mutex;

use crate::led_scheduler::LedScheduler;
use crate::led_system::LedSystem;
use crate::serial_manager::SerialManager;
use crate::subscribers::LedSubscribers;

lazy_static! {
    pub static ref LED_SYSTEM: Mutex<LedSystem> =
        Mutex::new(LedSystem::default());

    // NOTE: Need to call serial_manager!().setup() when the program starts!
    pub static ref SERIAL_MANAGER: Mutex<SerialManager> = Mutex::new(SerialManager::default());

    pub static ref STATE: Mutex<LedState> = Mutex::new(LedState::default());
    pub static ref SCHEDULE: Mutex<LedScheduler> =
        Mutex::new(LedScheduler::default());

    pub static ref SUBSCRIBERS: Mutex<LedSubscribers> = Mutex::new(LedSubscribers::new());
}

#[macro_export]
macro_rules! led_system {
    () => {
        crate::state::LED_SYSTEM.lock().unwrap()
    };
}

#[macro_export]
macro_rules! serial_manager {
    () => {
        crate::state::SERIAL_MANAGER.lock().unwrap()
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

#[macro_export]
macro_rules! subscribers {
    () => {
        crate::state::SUBSCRIBERS.try_lock().unwrap()
    };
}

#[derive(Default)]
pub struct LedState {
    changed_from_ui: bool,
    active: bool,
}

impl LedState {
    pub fn set_changed_from_ui(&mut self, value: bool) {
        self.changed_from_ui = value;
    }
    pub fn set_active(&mut self, value: bool) {
        self.active = value;
    }
    pub fn changed_from_ui(&self) -> bool {
        self.changed_from_ui
    }
    pub fn active(&self) -> bool {
        self.active
    }
}
