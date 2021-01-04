use serde_derive::{Deserialize, Serialize};

use crate::led_state::SERIAL_MANAGER;

/// Which rooms are currently active
#[derive(Eq, PartialEq, Hash, Serialize, Deserialize, Debug, Clone)]
pub enum Room {
    LivingRoom,
    Office,
    Bedroom,
}

/// Control which rooms are currently active
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RoomManager {
    pub living_room: bool,
    pub office: bool,
    pub bedroom: bool,
}

impl RoomManager {
    /// Set only this room to be active
    pub fn set_active_only(&mut self, room: Room) {
        self.living_room = false;
        self.office = false;
        self.bedroom = false;
        match room {
            Room::LivingRoom => self.living_room = true,
            Room::Office => self.office = true,
            Room::Bedroom => self.bedroom = true,
        }

        if let Ok(mut man) = SERIAL_MANAGER.get().write() {
            man.send_rooms(&self);
        }
    }

    pub fn set_active_rooms(&mut self, active_rooms: &Self) {
        *self = active_rooms.clone();
        if let Ok(mut man) = SERIAL_MANAGER.get().write() {
            man.send_rooms(&self);
        }
    }

    pub fn active_rooms(&self) -> &Self {
        &self
    }
}

impl Default for RoomManager {
    fn default() -> Self {
        Self {
            living_room: false,
            office: false,
            bedroom: false,
        }
    }
}
