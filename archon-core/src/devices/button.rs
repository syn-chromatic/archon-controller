#![allow(dead_code)]
#![allow(unused_variables)]

use crate::input::ButtonState;
use crate::input::InputButton;
use crate::input::InputType;

use embsys::crates::embassy_rp;
use embsys::devices::buttons;
use embsys::drivers::hardware;
use embsys::exts::std;

use std::time::Duration;

use buttons::standard::AdvButton;
use embassy_rp::gpio::AnyPin;
use hardware::get_pin;

#[derive(Copy, Clone)]
pub struct ButtonConfiguration {
    bounce_interval: Duration,
    repeat_interval: Duration,
    repeat_hold: Duration,
}

impl ButtonConfiguration {
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

pub struct DPadDevice {
    button: AdvButton,
    conf: ButtonConfiguration,
}

impl DPadDevice {
    pub fn new(pin: u8, conf: &ButtonConfiguration) -> Self {
        let button_pin: AnyPin = get_pin(pin);
        let button: AdvButton = AdvButton::new(
            button_pin,
            &conf.bounce_interval,
            &conf.repeat_interval,
            &conf.repeat_hold,
        );

        Self {
            button,
            conf: *conf,
        }
    }

    pub fn get_input(&mut self) -> Option<InputButton> {
        if self.button.is_pressed() {
            let state: ButtonState = ButtonState::from_adv_button(&mut self.button);
            let button: InputButton = InputButton::new(0, state);

            return Some(button);
        }

        None
    }

    pub fn get_input_as_type(&mut self) -> Option<InputType> {
        let input: Option<InputButton> = self.get_input();
        if let Some(input) = input {
            let input_type: InputType = InputType::Button(input);
            return Some(input_type);
        }
        None
    }
}
