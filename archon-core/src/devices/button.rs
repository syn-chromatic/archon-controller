#![allow(dead_code)]
#![allow(unused_variables)]

use crate::input::ButtonState;
use crate::input::InputButton;
use crate::input::InputType;

use embsys::crates::embassy_rp;
use embsys::crates::embassy_time;
use embsys::devices::buttons;
use embsys::drivers::hardware;
use embsys::exts::std;

use std::time::Duration as StdDuration;

use buttons::standard::AdvButton;
use embassy_rp::gpio::AnyPin;
use embassy_time::Duration;
use embassy_time::Instant;
use hardware::get_pin;

#[derive(Copy, Clone)]
pub struct ButtonConfiguration {
    bounce_interval: StdDuration,
    repeat_interval: StdDuration,
    repeat_hold: StdDuration,
}

impl ButtonConfiguration {
    pub fn new(
        bounce_interval: StdDuration,
        repeat_interval: StdDuration,
        repeat_hold: StdDuration,
    ) -> Self {
        Self {
            bounce_interval,
            repeat_interval,
            repeat_hold,
        }
    }
}

pub struct ButtonDevice {
    id: u8,
    button: AdvButton,
    press: Option<Instant>,
    conf: ButtonConfiguration,
}

impl ButtonDevice {
    fn press_duration(&mut self) -> Duration {
        if let Some(press) = self.press {
            return press.elapsed();
        } else {
            let press: Instant = Instant::now();
            let elapsed: Duration = press.elapsed();
            self.press = Some(press);
            return elapsed;
        }
    }
}

impl ButtonDevice {
    pub fn new(id: u8, pin: u8, conf: &ButtonConfiguration) -> Self {
        let button_pin: AnyPin = get_pin(pin);
        let button: AdvButton = AdvButton::new(
            button_pin,
            &conf.bounce_interval,
            &conf.repeat_interval,
            &conf.repeat_hold,
        );

        Self {
            id,
            button,
            press: None,
            conf: *conf,
        }
    }

    pub fn get_input(&mut self) -> Option<InputButton> {
        if self.button.is_pressed() {
            let duration: u16 = self.press_duration().as_millis() as u16;
            let state: ButtonState = ButtonState::new(true, duration);
            let button: InputButton = InputButton::new(self.id, state);
            return Some(button);
        }

        if !self.button.on_hold() || !self.button.on_repeat() {
            self.press = None;
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
