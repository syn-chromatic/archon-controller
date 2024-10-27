use crate::consts::MC_BUFFER;
use crate::endpoint::ArchonEndpoint;
use crate::endpoint::ArchonListenEndpoint;
use crate::utils::split_u16;

use embsys::crates::defmt;
use embsys::crates::embassy_net;
use embsys::exts::std;

use std::string::String;
use std::string::ToString;
use std::sync::Mutex;
use std::vec::Vec;

use defmt::Format;

use embassy_net::IpAddress;
use embassy_net::IpEndpoint;

#[derive(Clone, Format)]
pub struct DiscoveryInformation {
    remote_addr: [u8; 4],
    local_addr: [u8; 4],
    announce_info: AnnounceInformation,
}

impl DiscoveryInformation {
    pub fn new(
        remote_addr: [u8; 4],
        local_addr: [u8; 4],
        announce_info: AnnounceInformation,
    ) -> Self {
        Self {
            remote_addr,
            local_addr,
            announce_info,
        }
    }

    pub fn remote_addr(&self) -> [u8; 4] {
        self.remote_addr
    }

    pub fn remote_addr_type(&self) -> IpAddress {
        let (a0, a1, a2, a3) = self.remote_addr.into();
        IpAddress::v4(a0, a1, a2, a3)
    }

    pub fn local_addr(&self) -> [u8; 4] {
        self.local_addr
    }

    pub fn local_addr_type(&self) -> IpAddress {
        let (a0, a1, a2, a3) = self.local_addr.into();
        IpAddress::v4(a0, a1, a2, a3)
    }

    pub fn announce_info(&self) -> &AnnounceInformation {
        &self.announce_info
    }

    pub fn remote_tcp_endpoint(&self) -> IpEndpoint {
        let addr: IpAddress = self.remote_addr_type();
        let tcp_port: u16 = self.announce_info.tcp_port();
        let endpoint: IpEndpoint = IpEndpoint::new(addr, tcp_port);
        endpoint
    }

    pub fn defmt(&self) {
        defmt::info!(
            "Name: {} | Remote Addr: {:?}, Local Addr: {:?} | TCP Port: {}",
            self.announce_info.name,
            self.remote_addr,
            self.local_addr,
            self.announce_info.tcp_port,
        );
    }
}

#[derive(Clone, Format)]
pub struct AnnounceInformation {
    name: String,
    tcp_port: u16,
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
    pub fn new(name: &str, tcp_port: u16) -> Self {
        assert!(name.len() <= 32 && name.is_ascii());
        let name: String = name.to_string();
        Self { name, tcp_port }
    }

    pub fn from_buffer(buffer: &[u8; MC_BUFFER]) -> Self {
        let name: String = Self::ascii_be_to_string(&buffer[0..=31]);
        let tcp_port_be: [u8; 2] = buffer[32..=33].try_into().unwrap();
        let tcp_port: u16 = u16::from_be_bytes(tcp_port_be);
        Self { name, tcp_port }
    }

    pub fn to_buffer(&self) -> [u8; MC_BUFFER] {
        let mut buffer: [u8; MC_BUFFER] = [0; MC_BUFFER];

        let name_be: [u8; 32] = self.str_to_ascii_be();
        let tcp_port_be: [u8; 2] = split_u16(self.tcp_port);
        buffer[0..=31].copy_from_slice(&name_be);
        buffer[32..=33].copy_from_slice(&tcp_port_be);

        buffer
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn tcp_port(&self) -> u16 {
        self.tcp_port
    }
}

#[derive(Clone, Format)]
pub struct EstablishInformation {
    addr: [u8; 4],
    port: u16,
}

impl EstablishInformation {
    pub fn new(addr: [u8; 4], port: u16) -> Self {
        Self { addr, port }
    }

    pub fn from_buffer(buffer: &[u8; MC_BUFFER]) -> Self {
        let addr: [u8; 4] = buffer[0..=3].try_into().unwrap();
        let port_be: [u8; 2] = buffer[4..=5].try_into().unwrap();
        let port: u16 = u16::from_be_bytes(port_be);
        Self { addr, port }
    }

    pub fn to_buffer(&self) -> [u8; MC_BUFFER] {
        let mut buffer: [u8; MC_BUFFER] = [0; MC_BUFFER];

        let addr_be: [u8; 4] = self.addr;
        let port_be: [u8; 2] = split_u16(self.port);
        buffer[0..=3].copy_from_slice(&addr_be);
        buffer[4..=5].copy_from_slice(&port_be);

        buffer
    }

    pub fn addr(&self) -> [u8; 4] {
        self.addr
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn archon_endpoint(&self) -> ArchonEndpoint {
        ArchonEndpoint::new(self.addr.into(), self.port)
    }

    pub fn archon_listen_endpoint(&self) -> ArchonListenEndpoint {
        ArchonListenEndpoint::new(Some(self.addr.into()), self.port)
    }

    pub fn defmt(&self) {
        defmt::info!("ADDR: {:?} | PORT: {}", self.addr, self.port);
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
