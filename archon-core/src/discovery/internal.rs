use super::structures::AnnounceInformation;
use super::structures::DiscoveryInformation;
use super::structures::DiscoveryStatus;

use crate::consts::MC_BUFFER;

use embsys::crates::defmt;
use embsys::crates::embassy_futures;
use embsys::crates::embassy_net;
use embsys::crates::smoltcp;
use embsys::drivers::hardware;

use smoltcp::wire::IpVersion;

use embassy_net::udp::BindError;
use embassy_net::udp::PacketMetadata;
use embassy_net::udp::UdpSocket;
use embassy_net::IpAddress;
use embassy_net::IpEndpoint;

use hardware::WIFIController;

pub(in crate::discovery) static STATUS: DiscoveryStatus = DiscoveryStatus::new();

pub(in crate::discovery) async fn _udp_discovery() -> Result<(), BindError> {
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
    _udp_discovery_loop(&mut udp, &mut buf).await;
    Ok(())
}

pub(in crate::discovery) async fn _udp_discovery_loop(
    udp: &mut UdpSocket<'_>,
    buf: &mut [u8; MC_BUFFER],
) {
    loop {
        match udp.recv_from(buf).await {
            Ok((_n, _src)) => {
                defmt::info!("{:?}", buf);
                let addr: IpAddress = _src.endpoint.addr;

                match addr.version() {
                    IpVersion::Ipv4 => {
                        let addr: [u8; 4] = addr.as_bytes().try_into().unwrap();
                        let port: u16 = _src.endpoint.port;

                        let anno_info: AnnounceInformation = AnnounceInformation::from_buffer(&buf);
                        let disc_info: DiscoveryInformation =
                            DiscoveryInformation::new(anno_info, addr, port);
                        disc_info.defmt();

                        STATUS.push(disc_info);
                    }
                }
            }
            Err(e) => {
                defmt::info!("Error receiving message: {:?}", e);
            }
        }
    }
}

pub(in crate::discovery) async fn _udp_cancel_discovery() {
    while STATUS.state() {
        embassy_futures::yield_now().await;
    }
}
