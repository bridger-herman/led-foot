use std::net::TcpStream;

use tungstenite::{Message, WebSocket};

use crate::color::Color;

pub struct LedSubscribers {
    subscribers: Vec<WebSocket<TcpStream>>,
}

impl LedSubscribers {
    pub fn new() -> Self {
        Self {
            subscribers: vec![],
        }
    }

    pub fn add(&mut self, ws: WebSocket<TcpStream>) {
        self.subscribers.push(ws);
    }

    pub fn send_color_update(&mut self, current_color: &Color) {
        let mut to_remove = Vec::new();
        for (i, ws) in self.subscribers.iter_mut().enumerate() {
            if ws
                .write_message(Message::Text(
                    serde_json::to_string(current_color)
                        .expect("Failed to encode color"),
                ))
                .is_err()
            {
                to_remove.push(i);
            }
        }

        for ws_index in to_remove {
            self.subscribers.remove(ws_index);
        }
    }
}
