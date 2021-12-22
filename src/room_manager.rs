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
    pub living_room: Option<bool>,
    pub office: Option<bool>,
    pub bedroom: Option<bool>,
}

impl RoomManager {
    /// Set only this room to be active
    pub fn set_active_only(&mut self, room: Room) {
        self.living_room = Some(false);
        self.office = Some(false);
        self.bedroom = Some(false);
        match room {
            Room::LivingRoom => self.living_room = Some(true),
            Room::Office => self.office = Some(true),
            Room::Bedroom => self.bedroom = Some(true),
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
            living_room: Some(false),
            office: Some(false),
            bedroom: Some(false),
        }
    }
}
