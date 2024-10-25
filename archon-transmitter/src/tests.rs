use archon_core::devices::dpad::DPadConfiguration;
use archon_core::devices::dpad::DPadDevice;
use archon_core::devices::dpad::DPadPins;
use archon_core::devices::joystick::JoyStickAdc;
use archon_core::devices::joystick::JoyStickConfiguration;
use archon_core::devices::joystick::JoyStickCoordinate;
use archon_core::devices::joystick::JoyStickDevice;
use archon_core::devices::joystick::JoyStickFilter;
use archon_core::devices::polling::DevicePolling;
use archon_core::devices::rotary::RotaryAdc;
use archon_core::devices::rotary::RotaryConfiguration;
use archon_core::devices::rotary::RotaryDevice;
use archon_core::devices::rotary::RotaryFilter;
use archon_core::input::InputDPad;

use embsys::crates::defmt;
use embsys::crates::embassy_rp;
use embsys::crates::embassy_time;
use embsys::drivers;
use embsys::exts::std;

use std::time::Duration as StdDuration;

use drivers::hardware::HWController;

use embassy_rp::peripherals::*;
use embassy_time::Duration;

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
    let joystick_polling: DevicePolling = DevicePolling::new(Duration::from_millis(10));
    let mut joystick_conf: JoyStickConfiguration =
        JoyStickConfiguration::new(joystick_origin, joystick_polling);

    let ema_filter: JoyStickFilter = JoyStickFilter::ema(20);
    let interpolation_filter: JoyStickFilter =
        JoyStickFilter::linear_interpolation(10, 4070, 25, 4085);
    joystick_conf.add_filter(ema_filter);
    joystick_conf.add_filter(interpolation_filter);

    let mut joystick_device = JoyStickDevice::new(joystick_adc, joystick_conf);
    let _ = joystick_device.calibrate_center(5000).await;

    loop {
        let joystick_input = joystick_device.get_input().await;
        if let Ok(joystick_input) = joystick_input {
            if let Some(joystick_input) = joystick_input {
                defmt::info!("X: {:?} | Y: {:?}", joystick_input.x(), joystick_input.y());
            }
        }
    }
}

pub async fn rotary_test() {
    let v_pin = HWController::pac().PIN_28;
    let rotary_adc: RotaryAdc = RotaryAdc::new(v_pin);

    let rotary_polling: DevicePolling = DevicePolling::new(Duration::from_millis(10));
    let mut rotary_conf: RotaryConfiguration = RotaryConfiguration::new(rotary_polling);

    let ema_filter: RotaryFilter = RotaryFilter::ema(20);
    let interpolation_filter: RotaryFilter = RotaryFilter::linear_interpolation(40, 4080);
    rotary_conf.add_filter(ema_filter);
    rotary_conf.add_filter(interpolation_filter);

    let mut rotary_device: RotaryDevice = RotaryDevice::new(rotary_adc, rotary_conf);

    loop {
        let rotary_input = rotary_device.get_input().await;
        if let Ok(rotary_input) = rotary_input {
            if let Some(rotary_input) = rotary_input {
                let adc_value: u16 = rotary_input.value();
                defmt::info!("V: {:?}", adc_value);
            }
        }
    }
}

pub async fn button_test() {}
