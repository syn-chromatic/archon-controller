use super::input::InputASCII;
use super::input::InputType;
use crate::configuration::BUFFER;
use crate::diagnostics::FrameTime;

use embsys::crates::cortex_m;
use embsys::crates::defmt;
use embsys::crates::embassy_boot;
use embsys::crates::embassy_boot_rp;
use embsys::crates::embassy_executor;
use embsys::crates::embassy_futures;
use embsys::crates::embassy_net;
use embsys::crates::embassy_rp;
use embsys::crates::embassy_sync;
use embsys::crates::embassy_time;
use embsys::crates::embedded_storage_async;

use embsys::drivers;
use embsys::exts::non_std;
use embsys::helpers;

use non_std::buffer::ConcatBuffer;
use non_std::future::with_cancel;

use helpers::task_handler::TaskState;

use drivers::hardware::HWController;
use drivers::hardware::WIFIController;

use embassy_boot::FirmwareUpdaterError;
use embassy_rp::flash::Flash;
use embassy_rp::watchdog::Watchdog;
use embassy_rp::Peripherals;

use embassy_net::tcp::AcceptError;
use embassy_net::tcp::Error as TCPError;
use embassy_net::tcp::TcpSocket;
use embassy_net::IpAddress;
use embassy_net::IpListenEndpoint;

use embassy_time::with_timeout;
use embassy_time::Duration;
use embassy_time::Instant;
use embassy_time::TimeoutError;
use embassy_time::Timer;

pub struct ArchonReceiver {
    addr: Option<IpAddress>,
    port: u16,
    tcp_buffer: [u8; BUFFER],
}

impl ArchonReceiver {
    fn create_tcp_socket<'a>(
        rx_buffer: &'a mut [u8; BUFFER],
        tx_buffer: &'a mut [u8; BUFFER],
    ) -> TcpSocket<'a> {
        defmt::info!("Creating TCP Socket..");
        let tcp: TcpSocket = TcpSocket::new(WIFIController::stack_ref(), rx_buffer, tx_buffer);
        defmt::info!("TCP Socket created!");
        tcp
    }

    async fn accept_tcp_connection(&self, tcp: &mut TcpSocket<'_>) -> Result<(), AcceptError> {
        defmt::info!("Awaiting TCP Connection request..");
        let local_endpoint: IpListenEndpoint = self.get_local_endpoint();
        let result: Result<(), AcceptError> = tcp.accept(local_endpoint).await;
        if let Err(error) = result {
            self.defmt_tcp_accept_error(&error);
        } else {
            defmt::info!("TCP Connection accepted!");
        }
        result
    }

    fn get_local_endpoint(&self) -> IpListenEndpoint {
        let local_endpoint: IpListenEndpoint = IpListenEndpoint {
            addr: self.addr,
            port: self.port,
        };
        local_endpoint
    }

    fn tcp_result_empty(&self, tcp_result: usize) -> bool {
        if tcp_result == 0 {
            return true;
        }
        false
    }

    fn tcp_result_partial(&self, tcp_result: usize) -> bool {
        if tcp_result != BUFFER {
            return true;
        }
        false
    }

    async fn data_handler(&mut self, tcp_result: usize) -> bool {
        if self.tcp_result_empty(tcp_result) {
            return false;
        };

        if self.tcp_result_partial(tcp_result) {
            return false;
        }
        true
    }

    async fn read_data(&mut self, tcp: &mut TcpSocket<'_>) -> Result<usize, TCPError> {
        let fut = tcp.read(&mut self.tcp_buffer);
        let result: Result<Result<usize, TCPError>, TimeoutError> =
            with_timeout(Duration::from_secs(1), fut).await;

        if let Ok(result) = result {
            return result;
        }

        Err(TCPError::ConnectionReset)
    }

    async fn send_confirmation(&mut self, tcp: &mut TcpSocket<'_>) -> bool {
        let fut = tcp.write(&[0]);
        let result: Result<Result<usize, TCPError>, TimeoutError> =
            with_timeout(Duration::from_secs(1), fut).await;

        if let Ok(result) = result {
            return result.is_ok();
        }

        false
    }

    async fn flush_socket(
        &mut self,
        tcp: &mut TcpSocket<'_>,
    ) -> Result<Result<(), TCPError>, TimeoutError> {
        let fut = tcp.flush();
        let result: Result<Result<(), TCPError>, TimeoutError> =
            with_timeout(Duration::from_secs(4), fut).await;
        result
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

impl ArchonReceiver {
    pub fn new(addr: Option<IpAddress>, port: u16) -> Self {
        let tcp_buffer: [u8; BUFFER] = [0; BUFFER];

        Self {
            addr,
            port,
            tcp_buffer,
        }
    }

    pub async fn listen(&mut self) -> Result<(), AcceptError> {
        let mut frametime = FrameTime::new();

        let mut rx_buffer: [u8; BUFFER] = [0; BUFFER];
        let mut tx_buffer: [u8; BUFFER] = [0; BUFFER];
        let mut tcp: TcpSocket<'_> = Self::create_tcp_socket(&mut rx_buffer, &mut tx_buffer);

        self.accept_tcp_connection(&mut tcp).await?;

        loop {
            let instant = Instant::now();
            self.tcp_buffer.fill(0);
            let tcp_result = tcp.read(&mut self.tcp_buffer).await;

            if let Ok(tcp_result) = tcp_result {
                if tcp_result > 0 {
                    let input_type: InputType = InputType::from_buffer(&self.tcp_buffer);

                    match input_type {
                        InputType::DPad(_) => todo!(),
                        InputType::JoyStick(joystick) => {
                            let id = joystick.id();
                            let xy = joystick.xy();
                            defmt::info!(
                                "ID: {:?} | XY: {:?} | TCPBuffer: {:?} | TCPResult: {}",
                                id,
                                xy,
                                self.tcp_buffer,
                                tcp_result
                            );
                        }
                        InputType::ASCII(input_ascii) => {
                            let id = input_ascii.id();
                            let c = input_ascii.char();
                            defmt::info!(
                                "ID: {:?} | ASCII: {:?} | TCPBuffer: {:?} | TCPResult: {}",
                                id,
                                c,
                                self.tcp_buffer,
                                tcp_result
                            );
                        }
                        InputType::Rotary(_) => todo!(),
                    }

                    defmt::info!(
                        "TCPBuffer: {:?} | TCPResult: {}",
                        self.tcp_buffer,
                        tcp_result
                    );
                }
            } else if let Err(error) = tcp_result {
                self.defmt_tcp_error(error);
                break;
            }
            frametime.update(instant);
            frametime.defmt();
        }
        let _ = self.flush_socket(&mut tcp).await;
        Ok(())
    }
}

impl Drop for ArchonReceiver {
    fn drop(&mut self) {
        // self.tcp.close();
    }
}
