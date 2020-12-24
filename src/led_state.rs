use std::sync::RwLock;

use state::Storage;

use crate::room_manager::RoomManager;
use crate::serial_manager::SerialManager;
use crate::led_system::LedSystem;

pub static SERIAL_MANAGER: Storage<RwLock<SerialManager>> = Storage::new();
pub static ROOM_MANAGER: Storage<RwLock<RoomManager>> = Storage::new();
pub static LED_SYSTEM: Storage<RwLock<LedSystem>> = Storage::new();
pub static LED_STATE: Storage<RwLock<LedState>> = Storage::new();

pub fn init() {
    SERIAL_MANAGER.set(RwLock::new(SerialManager::default()));
    ROOM_MANAGER.set(RwLock::new(RoomManager::default()));
    LED_SYSTEM.set(RwLock::new(LedSystem::default()));
    LED_STATE.set(RwLock::new(LedState::default()));
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
