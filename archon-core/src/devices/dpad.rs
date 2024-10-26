#![allow(dead_code)]
#![allow(unused_variables)]

use crate::input::ButtonState;
use crate::input::DPad;
use crate::input::InputDPad;
use crate::input::InputType;

use embsys::crates::embassy_rp;
use embsys::crates::embassy_time;
use embsys::devices::buttons;
use embsys::drivers::hardware;
use embsys::exts::std;

use std::time::Duration as StdDuration;

use buttons::standard::AdvButton;

use hardware::get_pin;
use hardware::InputGPIO;
use hardware::InputTrait;

use embassy_rp::gpio::AnyPin;
use embassy_rp::gpio::Pin;
use embassy_rp::gpio::Pull;
use embassy_time::Duration;
use embassy_time::Instant;

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
    bounce_interval: StdDuration,
    repeat_interval: StdDuration,
    repeat_hold: StdDuration,
}

impl DPadConfiguration {
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

pub struct DPadButtons {
    up: AdvButton,
    right: AdvButton,
    down: AdvButton,
    left: AdvButton,
    up_press: Option<Instant>,
    right_press: Option<Instant>,
    down_press: Option<Instant>,
    left_press: Option<Instant>,
}

impl DPadButtons {
    fn up_press_duration(&mut self) -> Duration {
        if let Some(press) = self.up_press {
            return press.elapsed();
        } else {
            let press: Instant = Instant::now();
            let elapsed: Duration = press.elapsed();
            self.up_press = Some(press);
            return elapsed;
        }
    }

    fn right_press_duration(&mut self) -> Duration {
        if let Some(press) = self.right_press {
            return press.elapsed();
        } else {
            let press: Instant = Instant::now();
            let elapsed: Duration = press.elapsed();
            self.right_press = Some(press);
            return elapsed;
        }
    }

    fn down_press_duration(&mut self) -> Duration {
        if let Some(press) = self.down_press {
            return press.elapsed();
        } else {
            let press: Instant = Instant::now();
            let elapsed: Duration = press.elapsed();
            self.down_press = Some(press);
            return elapsed;
        }
    }

    fn left_press_duration(&mut self) -> Duration {
        if let Some(press) = self.left_press {
            return press.elapsed();
        } else {
            let press: Instant = Instant::now();
            let elapsed: Duration = press.elapsed();
            self.left_press = Some(press);
            return elapsed;
        }
    }

    fn up_press_reset(&mut self) {
        if !self.up.on_hold() || !self.up.on_repeat() {
            self.up_press = None;
        }
    }

    fn right_press_reset(&mut self) {
        if !self.right.on_hold() || !self.right.on_repeat() {
            self.right_press = None;
        }
    }

    fn down_press_reset(&mut self) {
        if !self.down.on_hold() || !self.down.on_repeat() {
            self.down_press = None;
        }
    }

    fn left_press_reset(&mut self) {
        if !self.left.on_hold() || !self.left.on_repeat() {
            self.left_press = None;
        }
    }
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
            up_press: None,
            right_press: None,
            down_press: None,
            left_press: None,
        }
    }
}

pub struct DPadDevice {
    id: u8,
    pins: DPadPins,
    conf: DPadConfiguration,
    buttons: DPadButtons,
}

impl DPadDevice {
    pub fn new(id: u8, pins: &DPadPins, conf: &DPadConfiguration) -> Self {
        let buttons: DPadButtons = DPadButtons::new(pins, conf);

        Self {
            id,
            pins: *pins,
            conf: *conf,
            buttons,
        }
    }

    pub fn get_inputs(&mut self) -> [Option<InputDPad>; 4] {
        let mut inputs: [Option<InputDPad>; 4] = [const { None }; 4];

        if self.buttons.up.is_pressed() {
            let duration: u16 = self.buttons.up_press_duration().as_millis() as u16;
            let state: ButtonState = ButtonState::new(true, duration);
            let dpad: InputDPad = InputDPad::new(self.id, DPad::Up, state);
            inputs[0] = Some(dpad);
        }

        if self.buttons.right.is_pressed() {
            let duration: u16 = self.buttons.right_press_duration().as_millis() as u16;
            let state: ButtonState = ButtonState::new(true, duration);
            let dpad: InputDPad = InputDPad::new(self.id, DPad::Right, state);
            inputs[0] = Some(dpad);
        }

        if self.buttons.down.is_pressed() {
            let duration: u16 = self.buttons.down_press_duration().as_millis() as u16;
            let state: ButtonState = ButtonState::new(true, duration);
            let dpad: InputDPad = InputDPad::new(self.id, DPad::Down, state);
            inputs[0] = Some(dpad);
        }

        if self.buttons.left.is_pressed() {
            let duration: u16 = self.buttons.left_press_duration().as_millis() as u16;
            let state: ButtonState = ButtonState::new(true, duration);
            let dpad: InputDPad = InputDPad::new(self.id, DPad::Left, state);
            inputs[0] = Some(dpad);
        }

        self.buttons.up_press_reset();
        self.buttons.right_press_reset();
        self.buttons.down_press_reset();
        self.buttons.left_press_reset();

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
