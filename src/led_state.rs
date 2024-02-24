use std::sync::RwLock;

use state::InitCell;

use crate::color::Color;
use crate::led_sequence::LedSequence;
use crate::led_config::LedConfig;
use crate::rooms::Rooms;
use crate::serial_manager::SerialManager;

pub static LED_STATE: InitCell<RwLock<LedState>> = InitCell::new();
pub static LED_CONFIG: InitCell<LedConfig> = InitCell::new();
pub static SERIAL_MANAGER: InitCell<RwLock<SerialManager>> = InitCell::new();

#[derive(Debug, Clone)]
pub struct LedState {
    /// Current color that the LEDs are
    pub current_color: Color,

    /// Current state of room relays
    pub current_rooms: Rooms,

    /// Current sequence the LEDs are running, if any
    pub current_sequence: Option<LedSequence>,

    /// Is the LED system currently running a sequence or transition?
    pub active: bool,

    /// Is the system in the process of shutting down?
    pub shutdown: bool,
}

pub fn init_global_state() {
    LED_STATE.set(RwLock::new(LedState {
        current_color: Color::new(0.0, 0.0, 0.0, 0.0),
        current_rooms: Rooms { living_room: false, office: false, bedroom: false },
        current_sequence: None,
        active: false,
        shutdown: false,
    }));

    LED_CONFIG.set(LedConfig::new());

    let mut mgr = SerialManager::new(&LED_CONFIG.get().tty_name);
    mgr.setup().expect("Unable to set up Serial Manager");
    SERIAL_MANAGER.set(RwLock::new(mgr));
}
