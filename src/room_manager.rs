use std::collections::HashMap;

use serde_derive::{Deserialize, Serialize};

/// Which rooms are currently active
#[derive(Eq, PartialEq, Hash, Serialize, Deserialize, Debug, Clone)]
pub enum Room {
    LivingRoom,
    Office,
    Bedroom,
}

/// Control which rooms are currently active
#[derive(Debug)]
pub struct RoomManager {
    active_rooms: HashMap<Room, bool>,
}

impl RoomManager {
    pub fn set_active_rooms(&mut self, active_rooms: HashMap<Room, bool>) {
        self.active_rooms = active_rooms;
        serial_manager!().send_rooms(&self.active_rooms);
    }

    pub fn active_rooms(&self) -> &HashMap<Room, bool> {
        &self.active_rooms
    }
}

impl Default for RoomManager {
    fn default() -> Self {
        let mut active_rooms = HashMap::new();
        active_rooms.insert(Room::LivingRoom, false);
        active_rooms.insert(Room::Office, false);
        active_rooms.insert(Room::Bedroom, false);

        Self { active_rooms }
    }
}
