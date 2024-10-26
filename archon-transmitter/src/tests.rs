use archon_core::devices::button::ButtonConfiguration;
use archon_core::devices::button::ButtonDevice;
use archon_core::devices::dpad::DPadConfiguration;
use archon_core::devices::dpad::DPadDevice;
use archon_core::devices::dpad::DPadPins;
use archon_core::devices::joystick::JoyStickAdc;
use archon_core::devices::joystick::JoyStickConfiguration;
use archon_core::devices::joystick::JoyStickCoordinate;
use archon_core::devices::joystick::JoyStickDevice;
use archon_core::devices::joystick::JoyStickFilter;
use archon_core::devices::layout::DeviceLayout;
use archon_core::devices::polling::DevicePolling;
use archon_core::devices::rotary::RotaryAdc;
use archon_core::devices::rotary::RotaryConfiguration;
use archon_core::devices::rotary::RotaryDevice;
use archon_core::devices::rotary::RotaryFilter;
use archon_core::input::ButtonState;
use archon_core::input::InputDPad;
use archon_core::input::InputType;

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
    let mut dpad_device: DPadDevice = DPadDevice::new(0, &dpad_pins, &dpad_conf);

    loop {
        let dpad_inputs: [Option<InputDPad>; 4] = dpad_device.get_inputs();
        for dpad_input in dpad_inputs {
            if let Some(dpad_input) = dpad_input {
                let dpad_state = dpad_input.state();
                let dpad_str: &str = dpad_input.dpad().as_str();
                defmt::info!(
                    "DPAD: {} | {:?}, {:?}",
                    dpad_str,
                    dpad_state.pressed(),
                    dpad_state.duration()
                );
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

    let mut joystick_device = JoyStickDevice::new(0, joystick_adc, joystick_conf);
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

    let mut rotary_device: RotaryDevice = RotaryDevice::new(0, rotary_adc, rotary_conf);

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

pub async fn button_test() {
    let bounce_interval: StdDuration = StdDuration::from_millis(10);
    let repeat_interval: StdDuration = StdDuration::from_millis(100);
    let repeat_hold: StdDuration = StdDuration::from_millis(500);
    let button_conf: ButtonConfiguration =
        ButtonConfiguration::new(bounce_interval, repeat_interval, repeat_hold);

    let mut button_device_1: ButtonDevice = ButtonDevice::new(0, 12, &button_conf);
    let mut button_device_2: ButtonDevice = ButtonDevice::new(0, 22, &button_conf);

    loop {
        let button_input = button_device_1.get_input();
        if let Some(button_input) = button_input {
            let button_state = button_input.state();
            defmt::info!(
                "B1: {:?}, {:?}",
                button_state.pressed(),
                button_state.duration()
            );
        }

        let button_input = button_device_2.get_input();
        if let Some(button_input) = button_input {
            let button_state = button_input.state();
            defmt::info!(
                "B2: {:?}, {:?}",
                button_state.pressed(),
                button_state.duration()
            );
        }
    }
}

fn create_dpad_device() -> DPadDevice {
    let bounce_interval: StdDuration = StdDuration::from_millis(10);
    let repeat_interval: StdDuration = StdDuration::from_millis(100);
    let repeat_hold: StdDuration = StdDuration::from_millis(500);

    let dpad_pins: DPadPins = DPadPins::new(10, 11, 15, 14);
    let dpad_conf: DPadConfiguration =
        DPadConfiguration::new(bounce_interval, repeat_interval, repeat_hold);
    let dpad_device: DPadDevice = DPadDevice::new(0, &dpad_pins, &dpad_conf);
    dpad_device
}

async fn create_joystick_device() -> JoyStickDevice {
    let x_pin = HWController::pac().PIN_26;
    let y_pin = HWController::pac().PIN_27;
    let joystick_adc: JoyStickAdc = JoyStickAdc::new(x_pin, y_pin);

    let joystick_origin: JoyStickCoordinate = JoyStickCoordinate::TopRight;
    let joystick_polling: DevicePolling = DevicePolling::new(Duration::from_millis(10));
    let mut joystick_conf: JoyStickConfiguration =
        JoyStickConfiguration::new(joystick_origin, joystick_polling);

    let ema_filter: JoyStickFilter = JoyStickFilter::ema(20);
    joystick_conf.add_filter(ema_filter);

    let mut joystick_device: JoyStickDevice = JoyStickDevice::new(0, joystick_adc, joystick_conf);
    let _ = joystick_device.calibrate_center(5000).await;

    joystick_device
}

fn create_joystick_button_device() -> ButtonDevice {
    let bounce_interval: StdDuration = StdDuration::from_millis(10);
    let repeat_interval: StdDuration = StdDuration::from_millis(100);
    let repeat_hold: StdDuration = StdDuration::from_millis(500);
    let button_conf: ButtonConfiguration =
        ButtonConfiguration::new(bounce_interval, repeat_interval, repeat_hold);

    let button_device: ButtonDevice = ButtonDevice::new(0, 22, &button_conf);
    button_device
}

fn create_l1_button_device() -> ButtonDevice {
    let bounce_interval: StdDuration = StdDuration::from_millis(10);
    let repeat_interval: StdDuration = StdDuration::from_millis(100);
    let repeat_hold: StdDuration = StdDuration::from_millis(500);
    let button_conf: ButtonConfiguration =
        ButtonConfiguration::new(bounce_interval, repeat_interval, repeat_hold);

    let button_device: ButtonDevice = ButtonDevice::new(1, 12, &button_conf);
    button_device
}

async fn create_rotary_device() -> RotaryDevice {
    let v_pin = HWController::pac().PIN_28;
    let rotary_adc: RotaryAdc = RotaryAdc::new(v_pin);

    let rotary_polling: DevicePolling = DevicePolling::new(Duration::from_millis(10));
    let mut rotary_conf: RotaryConfiguration = RotaryConfiguration::new(rotary_polling);

    let interpolation_filter: RotaryFilter = RotaryFilter::linear_interpolation(40, 4080);
    let ema_filter: RotaryFilter = RotaryFilter::ema(5);
    rotary_conf.add_filter(interpolation_filter);
    rotary_conf.add_filter(ema_filter);

    let rotary_device: RotaryDevice = RotaryDevice::new(0, rotary_adc, rotary_conf);
    rotary_device
}

pub async fn test_device_layout() {
    let mut layout: DeviceLayout = DeviceLayout::new();

    let dpad_device: DPadDevice = create_dpad_device();
    let joystick_device: JoyStickDevice = create_joystick_device().await;
    let joystick_button_device: ButtonDevice = create_joystick_button_device();
    let rotary_device: RotaryDevice = create_rotary_device().await;
    let l1_button_device: ButtonDevice = create_l1_button_device();

    layout.add_dpad(dpad_device);
    layout.add_joystick(joystick_device);
    layout.add_button(joystick_button_device);
    layout.add_rotary(rotary_device);
    layout.add_button(l1_button_device);

    loop {
        for input in layout.get_inputs().await {
            match input {
                InputType::DPad(input_dpad) => {
                    let id: u8 = input_dpad.id();
                    let dpad_str: &str = input_dpad.dpad().as_str();
                    let state: &ButtonState = input_dpad.state();

                    defmt::info!(
                        "ID: {} | DPAD: {} | STATE: {} | DURATION: {}",
                        id,
                        dpad_str,
                        state.pressed(),
                        state.duration()
                    );
                }
                InputType::JoyStick(input_joy_stick) => {
                    //
                }
                InputType::ASCII(input_ascii) => {}
                InputType::Rotary(input_rotary) => {
                    //
                }
                InputType::Button(input_button) => {
                    let id: u8 = input_button.id();
                    let state: &ButtonState = input_button.state();

                    defmt::info!(
                        "ID: {} | STATE: {} | DURATION: {}",
                        id,
                        state.pressed(),
                        state.duration()
                    );
                }
            }
        }
    }
}
