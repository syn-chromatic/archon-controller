use embsys::crates::embassy_net;

use embassy_net::udp::PacketMetadata;
use embassy_net::udp::UdpSocket;
use embassy_net::Stack;

pub struct UdpSocketWrapper<const S: usize> {
    rx_meta: [PacketMetadata; S],
    rx_buffer: [u8; S],
    tx_meta: [PacketMetadata; S],
    tx_buffer: [u8; S],
}

impl<const S: usize> UdpSocketWrapper<S> {
    pub fn new() -> Self {
        let rx_meta: [PacketMetadata; S] = [PacketMetadata::EMPTY; S];
        let rx_buffer: [u8; S] = [0; S];
        let tx_meta: [PacketMetadata; S] = [PacketMetadata::EMPTY; S];
        let tx_buffer: [u8; S] = [0; S];
        Self {
            rx_meta,
            rx_buffer,
            tx_meta,
            tx_buffer,
        }
    }

    pub fn socket<'a>(&'a mut self, stack: Stack<'a>) -> UdpSocket<'a> {
        UdpSocket::new(
            stack,
            &mut self.rx_meta,
            &mut self.rx_buffer,
            &mut self.tx_meta,
            &mut self.tx_buffer,
        )
    }
}
