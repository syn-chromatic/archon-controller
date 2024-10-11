#![allow(dead_code)]
#![allow(unused_variables)]

use archon_core::consts::TCP_BUFFER;
use archon_core::diagnostics::frametime::FrameTime;
use archon_core::endpoint::ArchonEndpoint;
use archon_core::input::InputType;
use archon_core::ring::RingBuffer;
use archon_core::status::ArchonStatus;

use embsys::crates::defmt;
use embsys::crates::embassy_net;
use embsys::crates::embassy_time;
use embsys::drivers;

use drivers::hardware::WIFIController;

use embassy_net::tcp::AcceptError;
use embassy_net::tcp::Error as TCPError;
use embassy_net::tcp::TcpSocket;
use embassy_net::IpListenEndpoint;

use embassy_time::with_timeout;
use embassy_time::Duration;
use embassy_time::Instant;
use embassy_time::TimeoutError;

pub struct ArchonReceiver<const INPUT_BUFFER: usize> {
    status: ArchonStatus,
    endpoint: ArchonEndpoint,
    ring: RingBuffer<InputType, INPUT_BUFFER>,
}

impl<const INPUT_BUFFER: usize> ArchonReceiver<INPUT_BUFFER> {
    fn create_socket<'a>(
        rx_buffer: &'a mut [u8; TCP_BUFFER],
        tx_buffer: &'a mut [u8; TCP_BUFFER],
    ) -> TcpSocket<'a> {
        defmt::info!("Creating TCP Socket..");
        let tcp: TcpSocket = TcpSocket::new(WIFIController::stack_ref(), rx_buffer, tx_buffer);
        defmt::info!("TCP Socket created!");
        tcp
    }

    async fn accept_socket(&mut self, tcp: &mut TcpSocket<'_>) -> Result<(), AcceptError> {
        defmt::info!("Awaiting TCP Connection request..");
        let endpoint: IpListenEndpoint = self.endpoint.endpoint();

        self.status.set_listening(true);
        let result: Result<(), AcceptError> = tcp.accept(endpoint).await;
        self.status.set_listening(false);

        if let Err(error) = result {
            self.defmt_tcp_accept_error(&error);
        } else {
            self.status.set_connected(true);
            defmt::info!("TCP Connection accepted!");
        }
        result
    }

    async fn flush_socket(
        &mut self,
        tcp: &mut TcpSocket<'_>,
    ) -> Result<Result<(), TCPError>, TimeoutError> {
        let fut = tcp.flush();
        let result: Result<Result<(), TCPError>, TimeoutError> =
            with_timeout(Duration::from_secs(1), fut).await;
        result
    }

    async fn receive_inputs(&mut self, tcp: &mut TcpSocket<'_>) {
        let mut frametime: FrameTime = FrameTime::new();

        loop {
            let instant: Instant = Instant::now();
            let result: Result<InputType, TCPError> = tcp.read_with(Self::read_input).await;

            if let Ok(input_type) = result {
                self.ring.add(input_type);
            } else if let Err(error) = result {
                self.status.set_connected(false);
                self.defmt_tcp_error(error);
                break;
            }
            frametime.update(instant);
            frametime.defmt();
        }
    }

    fn read_input(buffer: &mut [u8]) -> (usize, InputType) {
        defmt::info!("Len: {} | Buffer: {:?}", buffer.len(), buffer);
        let input_buffer: [u8; TCP_BUFFER] = buffer.try_into().unwrap();
        let input_type: InputType = InputType::from_buffer(&input_buffer);

        (buffer.len(), input_type)
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

impl<const INPUT_BUFFER: usize> ArchonReceiver<INPUT_BUFFER> {
    pub fn new() -> Self {
        let status: ArchonStatus = ArchonStatus::new();
        let endpoint: ArchonEndpoint = ArchonEndpoint::default();
        let ring: RingBuffer<InputType, INPUT_BUFFER> = RingBuffer::new();
        Self {
            status,
            endpoint,
            ring,
        }
    }

    pub async fn listen(&mut self) -> Result<(), AcceptError> {
        let mut rx_buffer: [u8; TCP_BUFFER] = [0; TCP_BUFFER];
        let mut tx_buffer: [u8; TCP_BUFFER] = [0; TCP_BUFFER];
        let mut tcp: TcpSocket<'_> = Self::create_socket(&mut rx_buffer, &mut tx_buffer);

        self.accept_socket(&mut tcp).await?;
        self.receive_inputs(&mut tcp).await;

        let _ = self.flush_socket(&mut tcp).await;
        self.ring.clear();
        Ok(())
    }

    pub fn disconnect(&self) {
        unimplemented!();
    }

    pub fn take(&mut self) -> Option<InputType> {
        self.ring.take()
    }

    pub fn set_endpoint(&mut self, endpoint: ArchonEndpoint) {
        self.endpoint = endpoint;
    }

    pub fn get_endpoint(&self) -> &ArchonEndpoint {
        &self.endpoint
    }

    pub fn get_status(&self) -> &ArchonStatus {
        &self.status
    }
}
