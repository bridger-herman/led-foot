use serde_derive::{Deserialize, Serialize};

/// Which rooms are currently active
#[derive(Eq, PartialEq, Hash, Serialize, Deserialize, Debug, Clone)]
pub enum Room {
    LivingRoom,
    Office,
    Bedroom,
}

/// Control which rooms are currently active
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Rooms {
    pub living_room: bool,
    pub office: bool,
    pub bedroom: bool,
}

impl Rooms {
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
    }

    pub fn set_active_rooms(&mut self, active_rooms: &Self) {
        *self = active_rooms.clone();
    }

    pub fn set_active_rooms_option(
        &mut self,
        active_rooms: &ScheduledRoomState,
    ) {
        *self = self.from_scheduled(active_rooms);
    }

    pub fn active_rooms(&self) -> &Self {
        &self
    }

    fn from_scheduled(&self, scheduled: &ScheduledRoomState) -> Rooms {
        let living_room = scheduled.living_room.unwrap_or(self.living_room);
        let office = scheduled.office.unwrap_or(self.office);
        let bedroom = scheduled.bedroom.unwrap_or(self.bedroom);
        Rooms {
            living_room,
            office,
            bedroom,
        }
    }
}

impl Default for Rooms {
    fn default() -> Self {
        Self {
            living_room: false,
            office: false,
            bedroom: false,
        }
    }
}

/// For scheduled events, allow rooms to be unset
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScheduledRoomState {
    pub living_room: Option<bool>,
    pub office: Option<bool>,
    pub bedroom: Option<bool>,
}
