use std::sync::RwLock;

use state::Storage;

use crate::led_scheduler::LedScheduler;
use crate::led_system::LedSystem;
use crate::room_manager::RoomManager;
use crate::serial_manager::SerialManager;

pub static SERIAL_MANAGER: Storage<RwLock<SerialManager>> = Storage::new();
pub static ROOM_MANAGER: Storage<RwLock<RoomManager>> = Storage::new();
pub static LED_SYSTEM: Storage<RwLock<LedSystem>> = Storage::new();
pub static LED_SCHEDULER: Storage<RwLock<LedScheduler>> = Storage::new();

/// Flag if the LEDs need to be interrupted to switch over to the next sequence
static SEQUENCE_INTERRUPT: Storage<RwLock<bool>> = Storage::new();

pub fn set_interrupt(value: bool) {
    if let Ok(mut interrupt) = SEQUENCE_INTERRUPT.get().write() {
        *interrupt = value;
    } else {
        error!("Unable to set interrupt!");
    }
}

pub fn is_interrupted() -> bool {
    if let Ok(interrupt) = SEQUENCE_INTERRUPT.get().read() {
        *interrupt
    } else {
        error!("Unable to get interrupt!");
        false
    }
}

pub fn init() {
    SEQUENCE_INTERRUPT.set(RwLock::new(false));
    SERIAL_MANAGER.set(RwLock::new(SerialManager::default()));
    ROOM_MANAGER.set(RwLock::new(RoomManager::default()));
    LED_SYSTEM.set(RwLock::new(LedSystem::default()));
    LED_SCHEDULER.set(RwLock::new(LedScheduler::default()));
}
