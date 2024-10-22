use crate::consts::MC_BUFFER;
use crate::endpoint::ArchonAddressIPv4;
use crate::endpoint::ArchonEndpoint;

use embsys::crates::defmt;
use embsys::exts::std;

use std::string::String;
use std::string::ToString;
use std::sync::Mutex;
use std::vec::Vec;

use defmt::Format;

#[derive(Clone, Format)]
pub struct AnnounceInformation {
    name: String,
}

impl AnnounceInformation {
    fn str_to_ascii_be(&self) -> [u8; 32] {
        let mut str_be: [u8; 32] = [0; 32];

        for (idx, char) in self.name.chars().enumerate() {
            str_be[idx] = char as u8;
        }
        str_be
    }

    fn ascii_be_to_string(buffer: &[u8]) -> String {
        let mut string: String = String::new();
        for value in buffer.iter() {
            string.push(*value as char);
        }
        string
    }
}

impl AnnounceInformation {
    pub fn new(name: &str) -> Self {
        assert!(name.len() <= 32 && name.is_ascii());
        let name: String = name.to_string();
        Self { name }
    }

    pub fn from_buffer(buffer: &[u8; MC_BUFFER]) -> Self {
        let name: String = Self::ascii_be_to_string(&buffer[0..=31]);
        Self { name }
    }

    pub fn to_buffer(&self) -> [u8; MC_BUFFER] {
        let mut buffer: [u8; MC_BUFFER] = [0; MC_BUFFER];

        let name_be: [u8; 32] = self.str_to_ascii_be();
        buffer[0..=31].copy_from_slice(&name_be);

        buffer
    }
}

#[derive(Clone, Format)]
pub struct DiscoveryInformation {
    info: AnnounceInformation,
    addr: [u8; 4],
    port: u16,
}

impl DiscoveryInformation {
    pub fn new(info: AnnounceInformation, addr: [u8; 4], port: u16) -> Self {
        Self { info, addr, port }
    }

    pub fn endpoint(&self) -> ArchonEndpoint {
        let addr: ArchonAddressIPv4 =
            ArchonAddressIPv4::new(self.addr[0], self.addr[1], self.addr[2], self.addr[3]);
        let endpoint: ArchonEndpoint = ArchonEndpoint::new(addr, self.port);
        endpoint
    }

    pub fn defmt(&self) {
        defmt::info!(
            "Name: {} | Addr: {:?} | Port: {}",
            self.info.name,
            self.addr,
            self.port
        );
    }
}

pub struct DiscoveryStatus {
    discovered: Mutex<Vec<DiscoveryInformation>>,
    state: Mutex<bool>,
}

impl DiscoveryStatus {
    pub const fn new() -> Self {
        let discovered: Mutex<Vec<DiscoveryInformation>> = Mutex::new(Vec::new());
        let state: Mutex<bool> = Mutex::new(false);
        Self { discovered, state }
    }

    pub fn discovered(&self) -> Vec<DiscoveryInformation> {
        let mut discovered: Vec<DiscoveryInformation> = Vec::new();
        for discovery in self.discovered.lock().iter() {
            discovered.push(discovery.clone());
        }
        discovered
    }

    pub fn state(&self) -> bool {
        *self.state.lock()
    }
}

impl DiscoveryStatus {
    pub(in crate::discovery) fn set_enabled(&self) {
        *self.state.lock() = true;
    }

    pub(in crate::discovery) fn set_disabled(&self) {
        *self.state.lock() = false;
    }

    pub(in crate::discovery) fn push(&self, info: DiscoveryInformation) {
        self.discovered.lock().push(info);
    }

    pub(in crate::discovery) fn clear(&self) {
        self.set_disabled();
        self.discovered.lock().clear();
    }
}
