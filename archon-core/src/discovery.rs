use crate::consts::MC_BUFFER;
use crate::endpoint::ArchonAddressIPv4;
use crate::endpoint::ArchonEndpoint;

use embsys::crates::defmt;
use embsys::crates::embassy_net;
use embsys::crates::smoltcp;
use embsys::drivers::hardware;
use embsys::exts::std;

use std::string::String;
use std::string::ToString;
use std::vec::Vec;

use smoltcp::wire::IpVersion;

use embassy_net::udp::BindError;
use embassy_net::udp::PacketMetadata;
use embassy_net::udp::SendError;
use embassy_net::udp::UdpSocket;
use embassy_net::IpAddress;
use embassy_net::IpEndpoint;

use embassy_net::MulticastError;

use hardware::WIFIController;

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
pub struct MultiCastDiscovery {
    multicast_addr: IpAddress,
}

impl MultiCastDiscovery {
    pub fn new() -> Self {
        let multicast_addr = IpAddress::v4(224, 0, 0, 1);
        Self { multicast_addr }
    }

    pub async fn join(&self) -> Result<bool, MulticastError> {
        WIFIController::stack_ref()
            .join_multicast_group(self.multicast_addr)
            .await
    }

    pub async fn discover(&self) -> Result<Vec<DiscoveryInformation>, BindError> {
        let mut rx_meta: [PacketMetadata; MC_BUFFER] = [PacketMetadata::EMPTY; MC_BUFFER];
        let mut rx_buffer: [u8; MC_BUFFER] = [0; MC_BUFFER];
        let mut tx_meta: [PacketMetadata; MC_BUFFER] = [PacketMetadata::EMPTY; MC_BUFFER];
        let mut tx_buffer: [u8; MC_BUFFER] = [0; MC_BUFFER];

        let mut udp: UdpSocket<'_> = UdpSocket::new(
            WIFIController::stack_ref(),
            &mut rx_meta,
            &mut rx_buffer,
            &mut tx_meta,
            &mut tx_buffer,
        );

        udp.bind(IpEndpoint::new(IpAddress::v4(0, 0, 0, 0), 5000))?;

        let mut buf = [0; MC_BUFFER];
        let mut discovered: Vec<DiscoveryInformation> = Vec::new();

        loop {
            match udp.recv_from(&mut buf).await {
                Ok((_n, _src)) => {
                    defmt::info!("{:?}", buf);
                    let addr: IpAddress = _src.endpoint.addr;

                    match addr.version() {
                        IpVersion::Ipv4 => {
                            let addr: [u8; 4] = addr.as_bytes().try_into().unwrap();
                            let port: u16 = _src.endpoint.port;

                            let anno_info: AnnounceInformation =
                                AnnounceInformation::from_buffer(&buf);
                            let disc_info: DiscoveryInformation =
                                DiscoveryInformation::new(anno_info, addr, port);
                            disc_info.defmt();

                            discovered.push(disc_info);

                            return Ok(discovered);
                        }
                    }
                }
                Err(e) => {
                    defmt::info!("Error receiving message: {:?}", e);
                }
            }
        }
    }

    pub async fn announce(&self, info: &AnnounceInformation) -> Result<(), SendError> {
        let mut rx_meta: [PacketMetadata; MC_BUFFER] = [PacketMetadata::EMPTY; MC_BUFFER];
        let mut rx_buffer: [u8; MC_BUFFER] = [0; MC_BUFFER];
        let mut tx_meta: [PacketMetadata; MC_BUFFER] = [PacketMetadata::EMPTY; MC_BUFFER];
        let mut tx_buffer: [u8; MC_BUFFER] = [0; MC_BUFFER];

        let mut udp: UdpSocket<'_> = UdpSocket::new(
            WIFIController::stack_ref(),
            &mut rx_meta,
            &mut rx_buffer,
            &mut tx_meta,
            &mut tx_buffer,
        );

        let endpoint: IpEndpoint = IpEndpoint::new(IpAddress::v4(0, 0, 0, 0), 5000);
        let _ = udp.bind(endpoint);

        let buffer = info.to_buffer();
        let endpoint: IpEndpoint = IpEndpoint::new(self.multicast_addr, 5000);
        udp.send_to(&buffer, endpoint).await?;

        Ok(())
    }
}
