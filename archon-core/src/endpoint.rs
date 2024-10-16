#![allow(dead_code)]
#![allow(unused_variables)]

use embsys::crates::embassy_net;

use embassy_net::IpAddress;
use embassy_net::IpEndpoint;
use embassy_net::IpListenEndpoint;
use embassy_net::Ipv4Address;

#[derive(Copy, Clone)]
pub struct ArchonAddressIPv4 {
    addr: IpAddress,
}

impl ArchonAddressIPv4 {
    pub fn new(a0: u8, a1: u8, a2: u8, a3: u8) -> Self {
        let addr: IpAddress = IpAddress::Ipv4(Ipv4Address::new(a0, a1, a2, a3));
        Self { addr }
    }

    pub fn addr(&self) -> &IpAddress {
        &self.addr
    }
}

pub struct ArchonListenEndpoint {
    addr: Option<ArchonAddressIPv4>,
    port: u16,
}

impl ArchonListenEndpoint {
    fn to_embassy_address(&self) -> Option<IpAddress> {
        if let Some(addr) = self.addr {
            return Some(addr.addr);
        }
        None
    }
}

impl ArchonListenEndpoint {
    pub const fn new(addr: Option<ArchonAddressIPv4>, port: u16) -> Self {
        Self { addr, port }
    }

    pub const fn default() -> Self {
        Self {
            addr: None,
            port: 0,
        }
    }

    pub fn set_addr(&mut self, addr: Option<ArchonAddressIPv4>) {
        self.addr = addr;
    }

    pub fn set_port(&mut self, port: u16) {
        self.port = port;
    }

    pub fn addr(&self) -> &Option<ArchonAddressIPv4> {
        &self.addr
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn endpoint(&self) -> IpListenEndpoint {
        IpListenEndpoint {
            addr: self.to_embassy_address(),
            port: self.port,
        }
    }
}

pub struct ArchonEndpoint {
    addr: ArchonAddressIPv4,
    port: u16,
}

impl ArchonEndpoint {
    pub const fn new(addr: ArchonAddressIPv4, port: u16) -> Self {
        Self { addr, port }
    }

    pub fn set_addr(&mut self, addr: ArchonAddressIPv4) {
        self.addr = addr;
    }

    pub fn set_port(&mut self, port: u16) {
        self.port = port;
    }

    pub fn addr(&self) -> &ArchonAddressIPv4 {
        &self.addr
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn endpoint(&self) -> IpEndpoint {
        IpEndpoint {
            addr: self.addr.addr,
            port: self.port,
        }
    }
}
