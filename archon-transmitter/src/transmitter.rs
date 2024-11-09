#![allow(dead_code)]

use archon_core::consts::UDP_BUFFER;
use archon_core::endpoint::ArchonEndpoint;
use archon_core::input::InputType;
use archon_core::socket::UdpSocketWrapper;

use embsys::crates::embassy_futures;
use embsys::crates::embassy_net;
use embsys::drivers;
use embsys::exts::non_std;
use embsys::exts::std;

use non_std::error::net::SocketError;
use std::sync::Mutex;
use std::sync::RwLock;
use std::sync::RwLockReadGuard;
use std::vec::Vec;

use drivers::hardware::WIFIController;

use embassy_net::udp::UdpSocket;
use embassy_net::IpEndpoint;

// Better to provide a method to initialize this manually by the user
static ARCHON: RwLock<ArchonTransmitter> = RwLock::new(ArchonTransmitter::new());

pub struct ArchonTransmitter {
    endpoint: Mutex<Option<ArchonEndpoint>>,
}

impl ArchonTransmitter {
    pub const fn new() -> Self {
        let endpoint: Mutex<Option<ArchonEndpoint>> = Mutex::new(None);

        Self { endpoint }
    }

    pub fn endpoint(&self) -> &Mutex<Option<ArchonEndpoint>> {
        &self.endpoint
    }

    pub fn set_endpoint(&self, endpoint: ArchonEndpoint) {
        *self.endpoint.lock() = Some(endpoint);
    }

    pub fn unset_endpoint(&self) {
        *self.endpoint.lock() = None;
    }

    pub fn setup_socket<'a>(
        &self,
        wrapper: &'a mut UdpSocketWrapper<UDP_BUFFER>,
    ) -> Result<(UdpSocket<'a>, IpEndpoint), SocketError> {
        if let Some(endpoint) = &*self.endpoint.lock() {
            let mut udp: UdpSocket<'_> = wrapper.socket(WIFIController::stack());
            let endpoint: IpEndpoint = endpoint.endpoint();
            udp.bind(endpoint.port)?;
            return Ok((udp, endpoint));
        }
        Err(SocketError::CustomError("Endpoint not set up"))
    }

    pub async fn send(
        &self,
        udp: &mut UdpSocket<'_>,
        endpoint: IpEndpoint,
        inputs: &Vec<InputType>,
    ) {
        for input in inputs {
            embassy_futures::yield_now().await;
            match input {
                InputType::DPad(input_dpad) => {
                    let buffer: _ = input_dpad.to_buffer();
                    let _ = udp.send_to(&buffer, endpoint).await;
                }
                InputType::JoyStick(input_joystick) => {
                    let buffer: _ = input_joystick.to_buffer();
                    let _ = udp.send_to(&buffer, endpoint).await;
                }
                InputType::ASCII(input_ascii) => {
                    let buffer: _ = input_ascii.to_buffer();
                    let _ = udp.send_to(&buffer, endpoint).await;
                }
                InputType::Rotary(input_rotary) => {
                    let buffer: _ = input_rotary.to_buffer();
                    let _ = udp.send_to(&buffer, endpoint).await;
                }
                InputType::Button(input_button) => {
                    let buffer: _ = input_button.to_buffer();
                    let _ = udp.send_to(&buffer, endpoint).await;
                }
            }
        }
    }
}

impl ArchonTransmitter {
    pub fn read_lock() -> RwLockReadGuard<'static, ArchonTransmitter> {
        ARCHON.read()
    }
}
