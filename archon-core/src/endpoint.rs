#![allow(dead_code)]
#![allow(unused_variables)]

use embsys::crates::embassy_net;

use embassy_net::IpAddress;
use embassy_net::IpListenEndpoint;

pub struct ArchonEndpoint {
    addr: Option<IpAddress>,
    port: u16,
}

impl ArchonEndpoint {
    pub fn new(addr: Option<IpAddress>, port: u16) -> Self {
        Self { addr, port }
    }

    pub fn default() -> Self {
        Self {
            addr: None,
            port: 0,
        }
    }

    pub fn set_addr(&mut self, addr: Option<IpAddress>) {
        self.addr = addr;
    }

    pub fn set_port(&mut self, port: u16) {
        self.port = port;
    }

    pub fn addr(&self) -> Option<IpAddress> {
        self.addr
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn endpoint(&self) -> IpListenEndpoint {
        IpListenEndpoint {
            addr: self.addr,
            port: self.port,
        }
    }
}
