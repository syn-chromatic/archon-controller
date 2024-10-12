use archon_core::consts::TCP_BUFFER;
use archon_core::endpoint::ArchonEndpoint;
use archon_core::input::DPad;
use archon_core::input::InputDPad;
use archon_core::input::InputType;
use archon_core::status::ArchonStatus;

use embsys::crates::defmt;
use embsys::crates::embassy_net;
use embsys::crates::embassy_rp;
use embsys::crates::embassy_time;
use embsys::devices::buttons;
use embsys::drivers;
use embsys::exts::std;

use std::sync::Mutex;
use std::sync::RwLock;
use std::sync::RwLockReadGuard;
use std::sync::RwLockWriteGuard;
use std::time::Duration as StdDuration;

use buttons::standard::AdvButton;
use drivers::hardware::get_pin;
use drivers::hardware::InputGPIO;
use drivers::hardware::InputTrait;
use drivers::hardware::WIFIController;

use embassy_rp::gpio::Pull;
use embassy_rp::peripherals::*;

use embassy_net::tcp::AcceptError;
use embassy_net::tcp::Error as TCPError;
use embassy_net::tcp::TcpSocket;
use embassy_net::IpListenEndpoint;

use embassy_time::with_timeout;
use embassy_time::Duration;
use embassy_time::Instant;
use embassy_time::TimeoutError;

pub struct ArchonTransmitter {
    endpoint: ArchonEndpoint,
}

impl ArchonTransmitter {
    fn print_incomplete_chunk(&self, chunk: usize, sent: usize) {
        defmt::info!(
            "Incomplete chunk sent! => Chunk: {} | Sent: {}",
            chunk,
            sent
        );
    }

    fn create_socket<'a>(
        rx_buffer: &'a mut [u8; TCP_BUFFER],
        tx_buffer: &'a mut [u8; TCP_BUFFER],
    ) -> TcpSocket<'a> {
        defmt::info!("Creating TCP Socket..");
        let tcp: TcpSocket = TcpSocket::new(WIFIController::stack_ref(), rx_buffer, tx_buffer);
        defmt::info!("TCP Socket created!");
        tcp
    }

    async fn accept_socket(&self, tcp: &mut TcpSocket<'_>) -> Result<(), AcceptError> {
        defmt::info!("Awaiting TCP Connection request..");
        let endpoint: IpListenEndpoint = self.endpoint.endpoint();
        let result: Result<(), AcceptError> = tcp.accept(endpoint).await;

        if let Err(error) = result {
            self.defmt_tcp_accept_error(&error);
        } else {
            defmt::info!("TCP Connection accepted!");
        }
        result
    }

    async fn flush_socket(
        &self,
        tcp: &mut TcpSocket<'_>,
    ) -> Result<Result<(), TCPError>, TimeoutError> {
        let fut = tcp.flush();
        let result: Result<Result<(), TCPError>, TimeoutError> =
            with_timeout(Duration::from_secs(1), fut).await;
        result
    }

    async fn send_input(&self, tcp: &mut TcpSocket<'_>) {
        let bounce_interval = StdDuration::from_millis(20);
        let repeat_interval = StdDuration::from_millis(100);
        let repeat_hold = StdDuration::from_millis(500);

        let mut button_1 = AdvButton::new(
            get_pin(10),
            &bounce_interval,
            &repeat_interval,
            &repeat_hold,
        );
        let mut button_2 = AdvButton::new(
            get_pin(11),
            &bounce_interval,
            &repeat_interval,
            &repeat_hold,
        );
        let mut button_3 = AdvButton::new(
            get_pin(14),
            &bounce_interval,
            &repeat_interval,
            &repeat_hold,
        );
        let mut button_4 = AdvButton::new(
            get_pin(15),
            &bounce_interval,
            &repeat_interval,
            &repeat_hold,
        );
        loop {
            if button_1.is_pressed() {
                let dpad = InputDPad::new(0, DPad::Left);
                let _ = tcp.write(&dpad.to_buffer()).await;
            } else if button_2.is_pressed() {
                let dpad = InputDPad::new(0, DPad::Up);
                let _ = tcp.write(&dpad.to_buffer()).await;
            } else if button_3.is_pressed() {
                let dpad = InputDPad::new(0, DPad::Right);
                let _ = tcp.write(&dpad.to_buffer()).await;
            } else if button_4.is_pressed() {
                let dpad = InputDPad::new(0, DPad::Down);
                let _ = tcp.write(&dpad.to_buffer()).await;
            }
        }
    }

    fn defmt_tcp_accept_error(&self, error: &AcceptError) {
        match error {
            AcceptError::InvalidState => defmt::info!("AcceptError: InvalidState"),
            AcceptError::InvalidPort => defmt::info!("AcceptError: InvalidPort"),
            AcceptError::ConnectionReset => defmt::info!("AcceptError: ConnectionReset"),
        }
    }

    fn defmt_tcp_error(&self, error: TCPError) {
        match error {
            TCPError::ConnectionReset => defmt::info!("Connecting Reset Error"),
        }
    }
}

impl ArchonTransmitter {
    pub fn new(endpoint: ArchonEndpoint) -> Self {
        Self { endpoint }
    }

    pub async fn run(&mut self) -> Result<(), AcceptError> {
        let mut rx_buffer: [u8; TCP_BUFFER] = [0; TCP_BUFFER];
        let mut tx_buffer: [u8; TCP_BUFFER] = [0; TCP_BUFFER];
        let mut tcp: TcpSocket<'_> = Self::create_socket(&mut rx_buffer, &mut tx_buffer);

        self.accept_socket(&mut tcp).await?;
        self.send_input(&mut tcp).await;
        let _ = self.flush_socket(&mut tcp).await;
        Ok(())
    }
}
