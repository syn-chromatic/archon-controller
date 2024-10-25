#![allow(dead_code)]
#![allow(unused_variables)]

use crate::input::DPad;
use crate::input::ButtonState;
use crate::input::InputDPad;
use crate::input::InputType;

use embsys::crates::embassy_rp;
use embsys::devices::buttons;
use embsys::drivers::hardware;
use embsys::exts::std;

use std::time::Duration;

use buttons::standard::AdvButton;

use hardware::get_pin;
use hardware::InputGPIO;
use hardware::InputTrait;

use embassy_rp::gpio::AnyPin;
use embassy_rp::gpio::Pin;
use embassy_rp::gpio::Pull;

pub struct DPadButton {
    vpin: u8,
    gpio: InputGPIO,
}

impl DPadButton {
    fn register_pin(pin: impl Pin) -> InputGPIO {
        let pull: Pull = Pull::Up;
        let gpio: InputGPIO = InputGPIO::new(pin, pull);
        gpio
    }
}

impl DPadButton {
    pub fn new(pin: impl Pin) -> Self {
        let vpin: u8 = pin.pin();
        let gpio: InputGPIO = Self::register_pin(pin);

        Self { vpin, gpio }
    }

    pub fn vpin(&self) -> u8 {
        self.vpin
    }

    pub fn is_pressed(&mut self) -> bool {
        let test = self.gpio.read();
        !test
    }
}

#[derive(Copy, Clone)]
pub struct DPadPins {
    up: u8,
    right: u8,
    down: u8,
    left: u8,
}

impl DPadPins {
    pub fn new(up: u8, right: u8, down: u8, left: u8) -> Self {
        Self {
            up,
            right,
            down,
            left,
        }
    }
}

#[derive(Copy, Clone)]
pub struct DPadConfiguration {
    bounce_interval: Duration,
    repeat_interval: Duration,
    repeat_hold: Duration,
}

impl DPadConfiguration {
    pub fn new(
        bounce_interval: Duration,
        repeat_interval: Duration,
        repeat_hold: Duration,
    ) -> Self {
        Self {
            bounce_interval,
            repeat_interval,
            repeat_hold,
        }
    }
}

pub struct DPadButtons {
    up: AdvButton,
    right: AdvButton,
    down: AdvButton,
    left: AdvButton,
}

impl DPadButtons {
    pub fn new(pins: &DPadPins, conf: &DPadConfiguration) -> Self {
        let up_pin: AnyPin = get_pin(pins.up);
        let right_pin: AnyPin = get_pin(pins.right);
        let down_pin: AnyPin = get_pin(pins.down);
        let left_pin: AnyPin = get_pin(pins.left);

        let up: AdvButton = AdvButton::new(
            up_pin,
            &conf.bounce_interval,
            &conf.repeat_interval,
            &conf.repeat_hold,
        );
        let right: AdvButton = AdvButton::new(
            right_pin,
            &conf.bounce_interval,
            &conf.repeat_interval,
            &conf.repeat_hold,
        );
        let down: AdvButton = AdvButton::new(
            down_pin,
            &conf.bounce_interval,
            &conf.repeat_interval,
            &conf.repeat_hold,
        );
        let left: AdvButton = AdvButton::new(
            left_pin,
            &conf.bounce_interval,
            &conf.repeat_interval,
            &conf.repeat_hold,
        );

        Self {
            up,
            right,
            down,
            left,
        }
    }
}

// pub struct DPadButtons {
//     up: DPadButton,
//     right: DPadButton,
//     down: DPadButton,
//     left: DPadButton,
// }

// impl DPadButtons {
//     pub fn new(pins: &DPadPins, conf: &DPadConfiguration) -> Self {
//         let up_pin: AnyPin = get_pin(pins.up);
//         let right_pin: AnyPin = get_pin(pins.right);
//         let down_pin: AnyPin = get_pin(pins.down);
//         let left_pin: AnyPin = get_pin(pins.left);

//         let up: DPadButton = DPadButton::new(up_pin);
//         let right: DPadButton = DPadButton::new(right_pin);
//         let down: DPadButton = DPadButton::new(down_pin);
//         let left: DPadButton = DPadButton::new(left_pin);

//         Self {
//             up,
//             right,
//             down,
//             left,
//         }
//     }
// }

pub struct DPadDevice {
    pins: DPadPins,
    conf: DPadConfiguration,
    buttons: DPadButtons,
}

impl DPadDevice {
    pub fn new(pins: &DPadPins, conf: &DPadConfiguration) -> Self {
        let buttons: DPadButtons = DPadButtons::new(pins, conf);

        Self {
            pins: *pins,
            conf: *conf,
            buttons,
        }
    }

    pub fn get_inputs(&mut self) -> [Option<InputDPad>; 4] {
        let mut inputs: [Option<InputDPad>; 4] = [const { None }; 4];

        if self.buttons.up.is_pressed() {
            let state: ButtonState = ButtonState::from_adv_button(&mut self.buttons.up);
            let dpad: InputDPad = InputDPad::new(0, DPad::Up, state);
            inputs[0] = Some(dpad);
        }

        if self.buttons.right.is_pressed() {
            let state: ButtonState = ButtonState::from_adv_button(&mut self.buttons.right);
            let dpad: InputDPad = InputDPad::new(0, DPad::Right, state);
            inputs[0] = Some(dpad);
        }

        if self.buttons.down.is_pressed() {
            let state: ButtonState = ButtonState::from_adv_button(&mut self.buttons.down);
            let dpad: InputDPad = InputDPad::new(0, DPad::Down, state);
            inputs[0] = Some(dpad);
        }

        if self.buttons.left.is_pressed() {
            let state: ButtonState = ButtonState::from_adv_button(&mut self.buttons.left);
            let dpad: InputDPad = InputDPad::new(0, DPad::Left, state);
            inputs[0] = Some(dpad);
        }

        inputs
    }

    pub fn get_inputs_as_types(&mut self) -> [Option<InputType>; 4] {
        let mut input_types: [Option<InputType>; 4] = [const { None }; 4];

        let inputs: [Option<InputDPad>; 4] = self.get_inputs();
        for (idx, dpad) in inputs.into_iter().enumerate() {
            if let Some(dpad) = dpad {
                input_types[idx] = Some(InputType::DPad(dpad));
            }
        }

        input_types
    }
}
