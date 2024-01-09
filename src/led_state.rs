use std::sync::RwLock;

use state::InitCell;

use crate::led_scheduler::LedScheduler;
use crate::led_system::LedSystem;
use crate::room_manager::RoomManager;
use crate::serial_manager::SerialManager;
use crate::wemo_manager::WemoManager;

pub static SERIAL_MANAGER: InitCell<RwLock<SerialManager>> = InitCell::new();
pub static ROOM_MANAGER: InitCell<RwLock<RoomManager>> = InitCell::new();
pub static LED_SYSTEM: InitCell<RwLock<LedSystem>> = InitCell::new();
pub static LED_SCHEDULER: InitCell<RwLock<LedScheduler>> = InitCell::new();
pub static WEMO_MANAGER: InitCell<WemoManager> = InitCell::new();

/// Flag if the LEDs need to be interrupted to switch over to the next sequence
static SEQUENCE_INTERRUPT: InitCell<RwLock<bool>> = InitCell::new();

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
    WEMO_MANAGER.set(WemoManager::new());
}
