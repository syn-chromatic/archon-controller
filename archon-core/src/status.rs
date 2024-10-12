#![allow(dead_code)]
#![allow(unused_variables)]

pub struct ArchonStatus {
    is_connected: bool,
    is_listening: bool,
}

impl ArchonStatus {
    pub const fn new() -> Self {
        Self {
            is_connected: false,
            is_listening: false,
        }
    }

    pub fn set_connected(&mut self, state: bool) {
        self.is_connected = state;
    }

    pub fn set_listening(&mut self, state: bool) {
        self.is_listening = state;
    }

    pub fn is_connected(&self) -> bool {
        self.is_connected
    }

    pub fn is_listening(&self) -> bool {
        self.is_listening
    }
}
