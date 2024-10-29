use super::internal::_udp_cancel_discovery;
use super::internal::_udp_discovery;
use super::internal::STATUS;
use super::structures::AnnounceInformation;
use super::structures::DiscoveryInformation;
use super::structures::DiscoveryStatus;
use super::structures::EstablishInformation;

use crate::consts::MC_BUFFER;

use embsys::crates::embassy_executor;
use embsys::crates::embassy_net;
use embsys::crates::embassy_time;
use embsys::drivers::hardware;
use embsys::exts::non_std;
use embsys::exts::std;

use non_std::error::net::SocketError;
use non_std::future::with_cancel;
use std::time::Duration;

use embassy_executor::SendSpawner;
use embassy_executor::SpawnError;
use embassy_time::with_timeout;
use embassy_time::Timer;

use embassy_net::tcp::Error;
use embassy_net::tcp::TcpSocket;
use embassy_net::udp::PacketMetadata;
use embassy_net::udp::SendError;
use embassy_net::udp::UdpSocket;
use embassy_net::IpAddress;
use embassy_net::IpEndpoint;
use embassy_net::IpListenEndpoint;
use embassy_net::MulticastError;

use hardware::WIFIController;

pub struct MultiCastDiscovery {
    multicast_addr: IpAddress,
    multicast_port: u16,
}

impl MultiCastDiscovery {
    fn multicast_endpoint(&self) -> IpEndpoint {
        IpEndpoint::new(self.multicast_addr, self.multicast_port)
    }
}

impl MultiCastDiscovery {
    pub fn new() -> Self {
        let multicast_addr: IpAddress = IpAddress::v4(230, 100, 80, 20);
        let multicast_port: u16 = 5000;
        Self {
            multicast_addr,
            multicast_port,
        }
    }

    pub async fn join(&self) -> Result<(), MulticastError> {
        WIFIController::stack_ref().join_multicast_group(self.multicast_addr)
    }

    pub async fn start_discovery(
        &self,
        spawner: &SendSpawner,
    ) -> Result<&DiscoveryStatus, SpawnError> {
        #[embassy_executor::task]
        async fn start_discovery_task() {
            STATUS.set_enabled();
            with_cancel(_udp_discovery(), _udp_cancel_discovery()).await;
            STATUS.clear();
        }

        spawner.spawn(start_discovery_task())?;
        Ok(&STATUS)
    }

    pub async fn stop_discovery(&self) {
        STATUS.set_disabled();
    }

    pub async fn connect(
        &self,
        discovery_info: &DiscoveryInformation,
    ) -> Result<EstablishInformation, SocketError> {
        let timeout: Duration = Duration::from_secs(5);

        let mut rx_buffer: [u8; MC_BUFFER] = [0; MC_BUFFER];
        let mut tx_buffer: [u8; MC_BUFFER] = [0; MC_BUFFER];

        let mut tcp: TcpSocket<'_> =
            TcpSocket::new(WIFIController::stack(), &mut rx_buffer, &mut tx_buffer);

        let tcp_endpoint: IpEndpoint = discovery_info.remote_tcp_endpoint();
        let establish: EstablishInformation =
            EstablishInformation::new(discovery_info.remote_addr(), 5000);

        with_timeout(timeout, tcp.connect(tcp_endpoint)).await??;
        with_timeout(timeout, tcp.write(&establish.to_buffer())).await??;
        with_timeout(timeout, tcp.flush()).await??;
        Timer::after_secs(1).await;
        Ok(establish)
    }

    pub async fn announce(&self) -> Result<EstablishInformation, SocketError> {
        let timeout: Duration = Duration::from_secs(5);

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

        let endpoint: IpListenEndpoint = IpListenEndpoint {
            addr: None,
            port: 0,
        };
        udp.bind(endpoint)?;

        let mut rx_buffer: [u8; MC_BUFFER] = [0; MC_BUFFER];
        let mut tx_buffer: [u8; MC_BUFFER] = [0; MC_BUFFER];
        let mut tcp: TcpSocket<'_> =
            TcpSocket::new(WIFIController::stack(), &mut rx_buffer, &mut tx_buffer);

        let tcp_port: u16 = 49586;
        let announce_info: AnnounceInformation =
            AnnounceInformation::new("RP2040 Receiver", tcp_port);
        let multicast_endpoint: IpEndpoint = self.multicast_endpoint();

        loop {
            let announce_buffer: [u8; MC_BUFFER] = announce_info.to_buffer();
            let result: Result<(), SendError> =
                udp.send_to(&announce_buffer, multicast_endpoint).await;

            if let Ok(_) = result {
                if let Ok(result) = with_timeout(timeout, tcp.accept(tcp_port)).await {
                    if let Ok(_) = result {
                        let mut buf: [u8; MC_BUFFER] = [0; MC_BUFFER];
                        let result: Result<usize, Error> =
                            with_timeout(timeout, tcp.read(&mut buf)).await?;
                        if let Ok(_size) = result {
                            let establish: EstablishInformation =
                                EstablishInformation::from_buffer(&buf);
                            establish.defmt();
                            with_timeout(timeout, tcp.flush()).await??;
                            Timer::after_secs(1).await;
                            return Ok(establish);
                        }
                    }
                } else {
                    continue;
                }
            }
            Timer::after_millis(1000).await;
        }
    }
}

impl Drop for MultiCastDiscovery {
    fn drop(&mut self) {
        STATUS.clear();
    }
}
