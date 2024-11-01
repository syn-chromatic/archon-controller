use embsys::drivers::hardware::HWController;
use embsys::drivers::hardware::WIFIController;
use embsys::exts::std;

use std::format;
use std::string::String;
use std::string::ToString;
use std::vec::Vec;

use embedded_menu::items::menu_item::SelectValue;
use embedded_menu::SelectValue;

use archon_core::input::DPad;
use archon_core::input::InputType;

#[derive(Copy, Clone)]
pub enum MainMenu {
    Discovery,
    Settings,
    Diagnostics,
}

#[derive(Copy, Clone, PartialEq, SelectValue)]
pub enum ButtonEnum {
    ON,
    OFF,
}

#[derive(Clone, PartialEq)]
pub struct U16Value {
    value: u16,
    value_str: String,
}

impl U16Value {
    pub fn new(value: u16) -> Self {
        Self {
            value,
            value_str: value.to_string(),
        }
    }
}

impl From<u16> for U16Value {
    fn from(value: u16) -> Self {
        U16Value::new(value)
    }
}

impl SelectValue for U16Value {
    fn marker(&self) -> &str {
        &self.value_str
    }
}

#[derive(Clone, PartialEq)]
pub struct F32Value {
    value: f32,
    value_str: String,
}

impl F32Value {
    pub fn new(value: f32) -> Self {
        Self {
            value,
            value_str: format!("{:.2}", value),
        }
    }
}

impl From<f32> for F32Value {
    fn from(value: f32) -> Self {
        F32Value::new(value)
    }
}

impl SelectValue for F32Value {
    fn marker(&self) -> &str {
        &self.value_str
    }
}

pub struct InputState {
    pub sys_voltage: F32Value,
    pub dpad_up: ButtonEnum,
    pub dpad_right: ButtonEnum,
    pub dpad_down: ButtonEnum,
    pub dpad_left: ButtonEnum,
    pub joystick_x: U16Value,
    pub joystick_y: U16Value,
    pub rotary: U16Value,
}

impl InputState {
    async fn get_sys_voltage() -> F32Value {
        WIFIController::control_mut().gpio_set(0, false).await;
        let sys_voltage: f32 = HWController::sys_voltage_blocking().unwrap();
        F32Value::new(sys_voltage)
    }
}

impl InputState {
    pub async fn from_inputs(inputs: &Vec<InputType>) -> Self {
        let sys_voltage: F32Value = Self::get_sys_voltage().await;
        let mut dpad_up: ButtonEnum = ButtonEnum::OFF;
        let mut dpad_right: ButtonEnum = ButtonEnum::OFF;
        let mut dpad_down: ButtonEnum = ButtonEnum::OFF;
        let mut dpad_left: ButtonEnum = ButtonEnum::OFF;
        let mut joystick_x: U16Value = U16Value::new(0);
        let mut joystick_y: U16Value = U16Value::new(0);
        let mut rotary: U16Value = U16Value::new(0);

        for input in inputs {
            match input {
                InputType::DPad(input_dpad) => match input_dpad.dpad() {
                    DPad::Up => dpad_up = ButtonEnum::ON,
                    DPad::Right => dpad_right = ButtonEnum::ON,
                    DPad::Down => dpad_down = ButtonEnum::ON,
                    DPad::Left => dpad_left = ButtonEnum::ON,
                },
                InputType::JoyStick(input_joy_stick) => {
                    joystick_x = input_joy_stick.x().into();
                    joystick_y = input_joy_stick.y().into();
                }
                InputType::ASCII(_input_ascii) => {}
                InputType::Rotary(input_rotary) => {
                    rotary = input_rotary.value().into();
                }
                InputType::Button(_input_button) => {}
            }
        }

        InputState {
            sys_voltage,
            dpad_up,
            dpad_right,
            dpad_down,
            dpad_left,
            joystick_x,
            joystick_y,
            rotary,
        }
    }
}
