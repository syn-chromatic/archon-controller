use super::internal::_udp_cancel_discovery;
use super::internal::_udp_discovery;
use super::internal::STATUS;
use super::structures::AnnounceInformation;
use super::structures::DiscoveryStatus;

use crate::consts::MC_BUFFER;

use embsys::crates::embassy_executor;
use embsys::crates::embassy_net;
use embsys::drivers::hardware;
use embsys::exts::non_std;

use non_std::future::with_cancel;

use embassy_executor::SendSpawner;
use embassy_executor::SpawnError;

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

    pub async fn announce(
        &self,
        info: &AnnounceInformation,
    ) -> Result<IpListenEndpoint, SendError> {
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

        let endpoint: IpEndpoint = IpEndpoint::new(IpAddress::v4(0, 0, 0, 0), 0);
        let _ = udp.bind(endpoint);

        let buffer = info.to_buffer();
        loop {
            let endpoint: IpEndpoint = IpEndpoint::new(self.multicast_addr, 5000);
            let result: Result<(), SendError> = udp.send_to(&buffer, endpoint).await;
            if let Ok(_) = result {
                let addr: IpListenEndpoint = udp.endpoint();
                return Ok(addr);
            }
        }
    }
}

impl Drop for MultiCastDiscovery {
    fn drop(&mut self) {
        STATUS.clear();
    }
}
