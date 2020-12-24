use std::sync::RwLock;

use state::Storage;

use crate::room_manager::RoomManager;
use crate::serial_manager::SerialManager;
use crate::led_system::LedSystem;

pub static SERIAL_MANAGER: Storage<RwLock<SerialManager>> = Storage::new();
pub static ROOM_MANAGER: Storage<RwLock<RoomManager>> = Storage::new();
pub static LED_SYSTEM: Storage<RwLock<LedSystem>> = Storage::new();

pub fn init() {
    SERIAL_MANAGER.set(RwLock::new(SerialManager::default()));
    ROOM_MANAGER.set(RwLock::new(RoomManager::default()));
    LED_SYSTEM.set(RwLock::new(LedSystem::default()));
}