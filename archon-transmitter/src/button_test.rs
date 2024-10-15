use archon_core::consts::TCP_BUFFER;
use archon_core::endpoint::ArchonEndpoint;
use archon_core::utils::u128_to_u16_max;
use archon_core::input::DPad;
use archon_core::input::DPadState;
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
use buttons::standard::Button;
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

pub async fn button_test() {
    let bounce_interval = StdDuration::from_millis(20);
    let repeat_interval = StdDuration::from_millis(100);
    let repeat_hold = StdDuration::from_millis(500);

    let mut button_1 = Button::new(get_pin(10));
    let mut button_2 = Button::new(get_pin(11));
    let mut button_3 = Button::new(get_pin(14));
    let mut button_4 = Button::new(get_pin(15));
    let mut button_5 = Button::new(get_pin(22));
    loop {
        if button_1.is_pressed() {
            defmt::info!("Left Button Pressed");
            WIFIController::control_mut().gpio_set(0, true).await;
        } else if button_2.is_pressed() {
            defmt::info!("Up Button Pressed");
            WIFIController::control_mut().gpio_set(0, true).await;
        } else if button_3.is_pressed() {
            defmt::info!("Right Button Pressed");
            WIFIController::control_mut().gpio_set(0, true).await;
        } else if button_4.is_pressed() {
            defmt::info!("Down Button Pressed");
            WIFIController::control_mut().gpio_set(0, true).await;
        } else if button_5.is_pressed() {
            defmt::info!("Down Button Pressed");
            WIFIController::control_mut().gpio_set(0, true).await;
        }
    }
}
