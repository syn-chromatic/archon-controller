#![allow(dead_code)]
#![allow(unused_variables)]

use crate::consts::INPUT_BUFFER;

use archon_core::consts::UDP_BUFFER;
use archon_core::endpoint::ArchonEndpoint;
use archon_core::input::InputType;
use archon_core::ring::RingBuffer;
use archon_core::status::ArchonStatus;

use embsys::crates::defmt;
use embsys::crates::embassy_net;
use embsys::drivers;
use embsys::exts::non_std;
use embsys::exts::std;

use non_std::error::net::UDPError;
use std::sync::Mutex;
use std::sync::RwLock;
use std::sync::RwLockReadGuard;
use std::sync::RwLockWriteGuard;

use drivers::hardware::WIFIController;

use embassy_net::udp::BindError;
use embassy_net::udp::PacketMetadata;
use embassy_net::udp::RecvError;
use embassy_net::udp::UdpMetadata;
use embassy_net::udp::UdpSocket;
use embassy_net::IpAddress;
use embassy_net::IpEndpoint;

// Better to provide a method to initialize this manually by the user
static ARCHON: RwLock<ArchonReceiver> = RwLock::new(ArchonReceiver::new());

pub struct ArchonReceiver {
    status: Mutex<ArchonStatus>,
    endpoint: Mutex<Option<ArchonEndpoint>>,
    ring: Mutex<RingBuffer<InputType, INPUT_BUFFER>>,
}

impl ArchonReceiver {
    fn create_socket<'a>(
        rx_meta: &'a mut [PacketMetadata; UDP_BUFFER],
        rx_buffer: &'a mut [u8; UDP_BUFFER],
        tx_meta: &'a mut [PacketMetadata; UDP_BUFFER],
        tx_buffer: &'a mut [u8; UDP_BUFFER],
    ) -> UdpSocket<'a> {
        defmt::info!("Creating UDP Socket..");
        let udp: UdpSocket = UdpSocket::new(
            WIFIController::stack_ref(),
            rx_meta,
            rx_buffer,
            tx_meta,
            tx_buffer,
        );
        defmt::info!("UDP Socket created!");
        udp
    }

    async fn receive_inputs(&self, udp: &mut UdpSocket<'_>) {
        let mut buffer: [u8; UDP_BUFFER] = [0; UDP_BUFFER];

        defmt::info!("Starting Receiving!");
        loop {
            defmt::info!("Receiving from buffer..");
            let result: Result<(usize, UdpMetadata), RecvError> = udp.recv_from(&mut buffer).await;

            if let Ok((size, _src)) = result {
                if size == UDP_BUFFER {
                    let input_type: InputType = InputType::from_buffer(&buffer);
                    self.ring.lock().add(input_type);
                }
            } else if let Err(error) = result {
                defmt::info!("Error: {:?}", error);
            }
        }
    }
}

impl ArchonReceiver {
    pub const fn new() -> Self {
        let status: ArchonStatus = ArchonStatus::new();
        let status: Mutex<ArchonStatus> = Mutex::new(status);
        let endpoint: Mutex<Option<ArchonEndpoint>> = Mutex::new(None);
        let ring: RingBuffer<InputType, INPUT_BUFFER> = RingBuffer::new();
        let ring: Mutex<RingBuffer<InputType, INPUT_BUFFER>> = Mutex::new(ring);
        Self {
            status,
            endpoint,
            ring,
        }
    }

    pub async fn listen(&self) -> Result<(), UDPError> {
        let mut rx_meta: [PacketMetadata; UDP_BUFFER] = [PacketMetadata::EMPTY; UDP_BUFFER];
        let mut rx_buffer: [u8; UDP_BUFFER] = [0; UDP_BUFFER];
        let mut tx_meta: [PacketMetadata; UDP_BUFFER] = [PacketMetadata::EMPTY; UDP_BUFFER];
        let mut tx_buffer: [u8; UDP_BUFFER] = [0; UDP_BUFFER];

        let mut udp: UdpSocket<'_> =
            Self::create_socket(&mut rx_meta, &mut rx_buffer, &mut tx_meta, &mut tx_buffer);

        if let Some(endpoint) = self.endpoint.lock().as_ref() {
            let address: IpAddress = endpoint.addr().addr().clone();
            let port: u16 = endpoint.port();
            defmt::info!("Addr: {:?} | Port: {}", address, port);
            udp.bind(IpEndpoint::new(address, port))?;

            self.receive_inputs(&mut udp).await;
            self.ring.lock().clear();
            return Ok(());
        }

        Err(UDPError::BindError(BindError::NoRoute))
    }

    pub fn disconnect(&self) {
        unimplemented!();
    }

    pub fn take(&self) -> Option<InputType> {
        self.ring.lock().take()
    }

    pub fn set_endpoint(&self, endpoint: ArchonEndpoint) {
        *self.endpoint.lock() = Some(endpoint);
    }

    pub fn get_endpoint(&self) -> &Mutex<Option<ArchonEndpoint>> {
        &self.endpoint
    }

    pub fn get_status(&self) -> &Mutex<ArchonStatus> {
        &self.status
    }
}

impl ArchonReceiver {
    pub fn read_lock<'a>() -> RwLockReadGuard<'a, ArchonReceiver> {
        ARCHON.read()
    }

    pub fn write_lock<'a>() -> RwLockWriteGuard<'a, ArchonReceiver> {
        ARCHON.write()
    }
}
