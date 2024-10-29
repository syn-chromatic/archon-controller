#![allow(dead_code)]
use crate::consts::INPUT_BUFFER;

use archon_core::consts::UDP_BUFFER;
use archon_core::devices::layout::DeviceLayout;
use archon_core::endpoint::ArchonEndpoint;
use archon_core::input::InputType;
use archon_core::ring::RingBuffer;

use embsys::crates::defmt;
use embsys::crates::embassy_futures;
use embsys::crates::embassy_net;
use embsys::drivers;
use embsys::exts::std;

use std::sync::Mutex;
use std::sync::RwLock;
use std::sync::RwLockReadGuard;
use std::sync::RwLockWriteGuard;
use std::vec::Vec;

use drivers::hardware::WIFIController;

use embassy_net::udp::PacketMetadata;
use embassy_net::udp::UdpSocket;
use embassy_net::IpEndpoint;

// Better to provide a method to initialize this manually by the user
static ARCHON: RwLock<ArchonTransmitter> = RwLock::new(ArchonTransmitter::new());

pub struct ArchonTransmitter {
    layout: Mutex<DeviceLayout>,
    endpoint: Mutex<Option<ArchonEndpoint>>,
    ring: Mutex<RingBuffer<InputType, INPUT_BUFFER>>,
}

impl ArchonTransmitter {
    fn create_socket<'a>(
        rx_meta: &'a mut [PacketMetadata; UDP_BUFFER],
        rx_buffer: &'a mut [u8; UDP_BUFFER],
        tx_meta: &'a mut [PacketMetadata; UDP_BUFFER],
        tx_buffer: &'a mut [u8; UDP_BUFFER],
    ) -> UdpSocket<'a> {
        defmt::info!("Creating UDP Socket..");
        let udp: UdpSocket = UdpSocket::new(
            WIFIController::stack(),
            rx_meta,
            rx_buffer,
            tx_meta,
            tx_buffer,
        );
        defmt::info!("UDP Socket created!");
        udp
    }

    async fn send_inputs(&self, udp: &mut UdpSocket<'_>) {
        if let Some(endpoint) = &*self.endpoint.lock() {
            let endpoint: IpEndpoint = endpoint.endpoint();
            let _ = udp.bind(endpoint.port);

            loop {
                embassy_futures::yield_now().await;
                let input = self.ring.lock().take();
                if let Some(input) = input {
                    match input {
                        InputType::DPad(input_dpad) => {
                            let buffer = input_dpad.to_buffer();
                            let _ = udp.send_to(&buffer, endpoint).await;
                        }
                        InputType::JoyStick(input_joystick) => {
                            let buffer = input_joystick.to_buffer();
                            let _ = udp.send_to(&buffer, endpoint).await;
                        }
                        InputType::ASCII(_input_ascii) => todo!(),
                        InputType::Rotary(input_rotary) => {
                            let buffer = input_rotary.to_buffer();
                            let _ = udp.send_to(&buffer, endpoint).await;
                        }
                        InputType::Button(input_button) => {
                            let buffer = input_button.to_buffer();
                            let _ = udp.send_to(&buffer, endpoint).await;
                        }
                    }
                }
            }
        }
    }
}

impl ArchonTransmitter {
    pub const fn new() -> Self {
        let layout: DeviceLayout = DeviceLayout::new();
        let layout: Mutex<DeviceLayout> = Mutex::new(layout);
        let endpoint: Mutex<Option<ArchonEndpoint>> = Mutex::new(None);
        let ring: RingBuffer<InputType, INPUT_BUFFER> = RingBuffer::new();
        let ring: Mutex<RingBuffer<InputType, INPUT_BUFFER>> = Mutex::new(ring);
        Self {
            layout,
            endpoint,
            ring,
        }
    }

    pub fn device_layout(&self) -> &Mutex<DeviceLayout> {
        &self.layout
    }

    pub fn endpoint(&self) -> &Mutex<Option<ArchonEndpoint>> {
        &self.endpoint
    }

    pub fn set_endpoint(&self, endpoint: ArchonEndpoint) {
        *self.endpoint.lock() = Some(endpoint);
    }

    pub async fn send(&self) -> Result<(), ()> {
        let mut rx_meta: [PacketMetadata; UDP_BUFFER] = [PacketMetadata::EMPTY; UDP_BUFFER];
        let mut rx_buffer: [u8; UDP_BUFFER] = [0; UDP_BUFFER];
        let mut tx_meta: [PacketMetadata; UDP_BUFFER] = [PacketMetadata::EMPTY; UDP_BUFFER];
        let mut tx_buffer: [u8; UDP_BUFFER] = [0; UDP_BUFFER];

        let mut udp: UdpSocket<'_> =
            Self::create_socket(&mut rx_meta, &mut rx_buffer, &mut tx_meta, &mut tx_buffer);
        self.send_inputs(&mut udp).await;

        Ok(())
    }

    pub async fn collect(&self) {
        loop {
            embassy_futures::yield_now().await;
            let inputs: Vec<InputType> = self.layout.lock().get_inputs().await;
            for input in inputs {
                self.ring.lock().add(input);
            }
        }
    }
}

impl ArchonTransmitter {
    pub fn read_lock<'a>() -> RwLockReadGuard<'a, ArchonTransmitter> {
        ARCHON.read()
    }

    pub fn write_lock<'a>() -> RwLockWriteGuard<'a, ArchonTransmitter> {
        ARCHON.write()
    }
}
