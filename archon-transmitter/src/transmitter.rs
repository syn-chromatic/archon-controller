use archon_core::consts::TCP_BUFFER;
use archon_core::devices::dpad::DPadConfiguration;
use archon_core::devices::dpad::DPadDevice;
use archon_core::devices::dpad::DPadPins;
use archon_core::endpoint::ArchonEndpoint;

use embsys::crates::defmt;
use embsys::crates::embassy_net;
use embsys::crates::embassy_time;
use embsys::drivers;
use embsys::exts::std;

use std::time::Duration as StdDuration;

use drivers::hardware::WIFIController;

use embassy_net::tcp::AcceptError;
use embassy_net::tcp::Error as TCPError;
use embassy_net::tcp::TcpSocket;
use embassy_net::IpListenEndpoint;

use embassy_time::with_timeout;
use embassy_time::Duration;
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
        let bounce_interval: StdDuration = StdDuration::from_millis(20);
        let repeat_interval: StdDuration = StdDuration::from_millis(100);
        let repeat_hold: StdDuration = StdDuration::from_millis(500);

        let dpad_pins: DPadPins = DPadPins::new(10, 11, 14, 15);
        let dpad_conf: DPadConfiguration =
            DPadConfiguration::new(bounce_interval, repeat_interval, repeat_hold);
        let mut dpad_device: DPadDevice = DPadDevice::new(&dpad_pins, &dpad_conf);

        loop {
            let dpad_inputs = dpad_device.get_inputs();
            for dpad_input in dpad_inputs {
                if let Some(dpad_input) = dpad_input {
                    let buffer = dpad_input.to_buffer();
                    defmt::info!("BUFFER: {:?}", buffer);
                    let _ = tcp.write(&buffer).await;
                }
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
