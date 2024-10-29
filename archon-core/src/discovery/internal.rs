use super::structures::AnnounceInformation;
use super::structures::DiscoveryInformation;
use super::structures::DiscoveryStatus;

use crate::consts::MC_BUFFER;

use embsys::crates::defmt;
use embsys::crates::embassy_futures;
use embsys::crates::embassy_net;
use embsys::drivers::hardware;

use embassy_net::udp::BindError;
use embassy_net::udp::PacketMetadata;
use embassy_net::udp::UdpSocket;
use embassy_net::IpAddress;
use embassy_net::IpEndpoint;
use embassy_net::StaticConfigV4;

use hardware::WIFIController;

pub(in crate::discovery) static STATUS: DiscoveryStatus = DiscoveryStatus::new();

pub(in crate::discovery) fn _get_local_addr() -> [u8; 4] {
    let config: Option<StaticConfigV4> = WIFIController::borrow_mut().get_config_v4();
    if let Some(config) = config {
        let addr: [u8; 4] = config.address.address().octets();
        return addr;
    }
    [0, 0, 0, 0]
}

pub(in crate::discovery) async fn _udp_discovery() -> Result<(), BindError> {
    let mut rx_meta: [PacketMetadata; MC_BUFFER] = [PacketMetadata::EMPTY; MC_BUFFER];
    let mut rx_buffer: [u8; MC_BUFFER] = [0; MC_BUFFER];
    let mut tx_meta: [PacketMetadata; MC_BUFFER] = [PacketMetadata::EMPTY; MC_BUFFER];
    let mut tx_buffer: [u8; MC_BUFFER] = [0; MC_BUFFER];

    let mut udp: UdpSocket<'_> = UdpSocket::new(
        WIFIController::stack(),
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

                match addr {
                    IpAddress::Ipv4(ipv4_addr) => {
                        let remote_addr: [u8; 4] = ipv4_addr.octets();
                        let local_addr: [u8; 4] = _get_local_addr();

                        let announce_info: AnnounceInformation =
                            AnnounceInformation::from_buffer(&buf);
                        let discovery_info: DiscoveryInformation =
                            DiscoveryInformation::new(remote_addr, local_addr, announce_info);

                        STATUS.push(discovery_info);
                    }
                    IpAddress::Ipv6(_ipv6_addr) => todo!(),
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
    defmt::info!("Canceled UDP Discovery!");
}
