#![allow(dead_code)]
#![allow(unused_variables)]

use super::input::InputType;
use crate::configuration::TCP_BUFFER;
use crate::diagnostics::FrameTime;

use embsys::crates::defmt;
use embsys::crates::embassy_net;
use embsys::crates::embassy_time;
use embsys::drivers;

use drivers::hardware::WIFIController;

use embassy_net::tcp::AcceptError;
use embassy_net::tcp::Error as TCPError;
use embassy_net::tcp::TcpSocket;
use embassy_net::IpAddress;
use embassy_net::IpListenEndpoint;

use embassy_time::with_timeout;
use embassy_time::Duration;
use embassy_time::Instant;
use embassy_time::TimeoutError;

struct RingBuffer<T, const SIZE: usize> {
    buffer: [Option<T>; SIZE],
    head: usize,
    tail: usize,
    is_full: bool,
}

impl<T, const SIZE: usize> RingBuffer<T, SIZE> {
    fn new() -> Self {
        assert!(SIZE > 0, "RingBuffer size must be greater than 0");
        RingBuffer {
            buffer: [const { None }; SIZE],
            head: 0,
            tail: 0,
            is_full: false,
        }
    }

    fn add(&mut self, item: T) {
        self.buffer[self.head] = Some(item);
        self.head = (self.head + 1) % SIZE;

        if self.is_full {
            self.tail = (self.tail + 1) % SIZE;
        }

        self.is_full = self.head == self.tail;
    }

    fn take(&mut self) -> Option<T> {
        if self.head == self.tail && !self.is_full {
            return None;
        }

        let item: Option<T> = self.buffer[self.tail].take();
        self.tail = (self.tail + 1) % SIZE;
        self.is_full = false;

        item
    }

    fn clear(&mut self) {
        self.head = 0;
        self.tail = 0;
        self.is_full = false;

        for i in 0..SIZE {
            self.buffer[i] = None;
        }
    }

    fn is_empty(&self) -> bool {
        self.head == self.tail && !self.is_full
    }

    fn is_full(&self) -> bool {
        self.is_full
    }
}

pub struct ArchonStatus {
    is_connected: bool,
    is_listening: bool,
}

impl ArchonStatus {
    pub fn new() -> Self {
        Self {
            is_connected: false,
            is_listening: false,
        }
    }

    pub fn set_connected(&mut self, state: bool) {
        self.is_connected = state;
    }

    pub fn set_listening(&mut self, state: bool) {
        self.is_listening = state;
    }

    pub fn is_connected(&self) -> bool {
        self.is_connected
    }

    pub fn is_listening(&self) -> bool {
        self.is_listening
    }
}

pub struct ArchonEndpoint {
    addr: Option<IpAddress>,
    port: u16,
}

impl ArchonEndpoint {
    pub fn new(addr: Option<IpAddress>, port: u16) -> Self {
        Self { addr, port }
    }

    pub fn default() -> Self {
        Self {
            addr: None,
            port: 0,
        }
    }

    pub fn set_addr(&mut self, addr: Option<IpAddress>) {
        self.addr = addr;
    }

    pub fn set_port(&mut self, port: u16) {
        self.port = port;
    }

    pub fn addr(&self) -> Option<IpAddress> {
        self.addr
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn endpoint(&self) -> IpListenEndpoint {
        IpListenEndpoint {
            addr: self.addr,
            port: self.port,
        }
    }
}

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
