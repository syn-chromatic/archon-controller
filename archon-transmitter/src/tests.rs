use archon_core::consts::UDP_BUFFER;
use archon_core::devices::dpad::DPadButtons;
use archon_core::devices::dpad::DPadConfiguration;
use archon_core::devices::dpad::DPadDevice;
use archon_core::devices::dpad::DPadPins;
use archon_core::devices::joystick::JoyStickAdc;
use archon_core::devices::joystick::JoyStickConfiguration;
use archon_core::devices::joystick::JoyStickCoordinate;
use archon_core::devices::joystick::JoyStickDevice;
use archon_core::devices::joystick::JoyStickFilter;
use archon_core::devices::polling::DevicePolling;
use archon_core::endpoint::ArchonListenEndpoint;
use archon_core::input::DPad;
use archon_core::input::DPadState;
use archon_core::input::InputDPad;
use archon_core::input::InputType;
use archon_core::status::ArchonStatus;
use archon_core::utils::u128_to_u16_max;

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
use drivers::hardware::HWController;
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
use embassy_time::Timer;

pub fn dpad_test() {
    let bounce_interval: StdDuration = StdDuration::from_millis(10);
    let repeat_interval: StdDuration = StdDuration::from_millis(100);
    let repeat_hold: StdDuration = StdDuration::from_millis(500);

    let dpad_pins: DPadPins = DPadPins::new(10, 11, 15, 14);
    let dpad_conf: DPadConfiguration =
        DPadConfiguration::new(bounce_interval, repeat_interval, repeat_hold);
    let mut dpad_device: DPadDevice = DPadDevice::new(&dpad_pins, &dpad_conf);

    loop {
        let dpad_inputs: [Option<InputDPad>; 4] = dpad_device.get_inputs();
        for dpad_input in dpad_inputs {
            if let Some(dpad_input) = dpad_input {
                let dpad_str: &str = dpad_input.dpad().as_str();
                let buffer = dpad_input.to_buffer();
                defmt::info!("DPAD: {} | BUFFER: {:?}", dpad_str, buffer);
            }
        }
    }
}

pub async fn joystick_test() {
    let x_pin: PIN_26 = HWController::pac().PIN_26;
    let y_pin: PIN_27 = HWController::pac().PIN_27;
    let joystick_adc: JoyStickAdc = JoyStickAdc::new(x_pin, y_pin);

    let joystick_origin: JoyStickCoordinate = JoyStickCoordinate::TopRight;
    let joystick_filter: JoyStickFilter = JoyStickFilter::ema(20);
    let joystick_polling: DevicePolling = DevicePolling::new(Duration::from_millis(10));
    let joystick_conf: JoyStickConfiguration =
        JoyStickConfiguration::new(joystick_origin, joystick_filter, joystick_polling);

    let mut joystick_device = JoyStickDevice::new(joystick_adc, joystick_conf);
    let _ = joystick_device.calibrate_center(5000).await;

    loop {
        let joystick_input = joystick_device.get_input().await;
        if let Ok(joystick_input) = joystick_input {
            // let buffer = joystick_input.to_buffer();
            // defmt::info!("BUFFER: {:?}", buffer);
            if let Some(joystick_input) = joystick_input {
                defmt::info!("X: {:?} | Y: {:?}", joystick_input.x(), joystick_input.y());
            }
            // Timer::after_millis(100).await;
        }
    }
}
