#![allow(unused_imports)]

use super::enums::ButtonEnum;

use embsys::crates::embassy_rp;
use embsys::drivers::hardware::HWController;
use embsys::drivers::hardware::WIFIController;
use embsys::exts::std;

use std::format;
use std::string::String;
use std::string::ToString;
use std::vec::Vec;

use embassy_rp::adc::Error as AdcError;

use embedded_menu::items::menu_item::SelectValue;
use embedded_menu::items::MenuItem;

use archon_core::input::DPad;
use archon_core::input::InputType;

#[derive(Clone, PartialEq)]
pub struct SelectString {
    string: String,
}

impl SelectString {
    pub fn new(string: String) -> Self {
        Self { string }
    }
}

impl SelectValue for SelectString {
    fn marker(&self) -> &str {
        &self.string
    }
}

impl From<String> for SelectString {
    fn from(value: String) -> Self {
        SelectString::new(value)
    }
}

impl From<&str> for SelectString {
    fn from(value: &str) -> Self {
        SelectString::new(value.to_string())
    }
}

#[derive(Copy, Clone, PartialEq)]
pub struct SubMenuSelect {
    index: Option<usize>,
}

impl SubMenuSelect {
    pub fn new(index: usize) -> Self {
        Self { index: Some(index) }
    }

    pub fn index(&self) -> Option<usize> {
        self.index
    }
}

impl Default for SubMenuSelect {
    fn default() -> Self {
        Self { index: None }
    }
}

impl SelectValue for SubMenuSelect {
    fn marker(&self) -> &str {
        ">"
    }
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

#[derive(Clone, PartialEq)]
pub enum InputStateEnum {
    F32(F32Value),
    U16(U16Value),
    Button(ButtonEnum),
}

impl InputStateEnum {
    pub fn u16(value: u16) -> Self {
        InputStateEnum::U16(U16Value::new(value))
    }

    pub fn f32(value: f32) -> Self {
        InputStateEnum::F32(F32Value::new(value))
    }

    pub fn button(value: bool) -> Self {
        InputStateEnum::Button(ButtonEnum::new(value))
    }
}

impl SelectValue for InputStateEnum {
    fn marker(&self) -> &str {
        match self {
            InputStateEnum::F32(f32_value) => f32_value.marker(),
            InputStateEnum::U16(u16_value) => u16_value.marker(),
            InputStateEnum::Button(button_enum) => button_enum.marker(),
        }
    }
}

pub struct InputState {
    pub sys_voltage: InputStateEnum,
    pub dpad_up: InputStateEnum,
    pub dpad_right: InputStateEnum,
    pub dpad_down: InputStateEnum,
    pub dpad_left: InputStateEnum,
    pub joystick_x: InputStateEnum,
    pub joystick_y: InputStateEnum,
    pub rotary: InputStateEnum,
}

impl InputState {
    async fn get_sys_voltage() -> Result<InputStateEnum, AdcError> {
        // Causes hang when WI-FI task completes?
        // Needed to disable LED to get accurate sys voltage
        // As CYW43 is connected to GP29
        // WIFIController::control_mut().gpio_set(0, false).await;

        let sys_voltage: f32 = HWController::sys_voltage().await?;
        Ok(InputStateEnum::f32(sys_voltage))
    }
}

impl InputState {
    pub async fn from_inputs(inputs: &Vec<InputType>) -> Result<Self, AdcError> {
        let sys_voltage: InputStateEnum = Self::get_sys_voltage().await?;
        let mut dpad_up: InputStateEnum = InputStateEnum::button(false);
        let mut dpad_right: InputStateEnum = InputStateEnum::button(false);
        let mut dpad_down: InputStateEnum = InputStateEnum::button(false);
        let mut dpad_left: InputStateEnum = InputStateEnum::button(false);
        let mut joystick_x: InputStateEnum = InputStateEnum::u16(0);
        let mut joystick_y: InputStateEnum = InputStateEnum::u16(0);
        let mut rotary: InputStateEnum = InputStateEnum::u16(0);

        for input in inputs {
            match input {
                InputType::DPad(input_dpad) => match input_dpad.dpad() {
                    DPad::Up => dpad_up = InputStateEnum::button(true),
                    DPad::Right => dpad_right = InputStateEnum::button(true),
                    DPad::Down => dpad_down = InputStateEnum::button(true),
                    DPad::Left => dpad_left = InputStateEnum::button(true),
                },
                InputType::JoyStick(input_joy_stick) => {
                    joystick_x = InputStateEnum::u16(input_joy_stick.x());
                    joystick_y = InputStateEnum::u16(input_joy_stick.y());
                }
                InputType::ASCII(_input_ascii) => {}
                InputType::Rotary(input_rotary) => {
                    rotary = InputStateEnum::u16(input_rotary.value());
                }
                InputType::Button(_input_button) => {}
            }
        }

        Ok(InputState {
            sys_voltage,
            dpad_up,
            dpad_right,
            dpad_down,
            dpad_left,
            joystick_x,
            joystick_y,
            rotary,
        })
    }

    pub fn to_menu_items(&self) -> Vec<MenuItem<&str, (), InputStateEnum, true>> {
        let mut items: Vec<MenuItem<&str, (), InputStateEnum, true>> = Vec::new();

        items.push(MenuItem::new(" SYS VOLTAGE", self.sys_voltage.clone()));
        items.push(MenuItem::new(" DPAD UP", self.dpad_up.clone()));
        items.push(MenuItem::new(" DPAD RIGHT", self.dpad_right.clone()));
        items.push(MenuItem::new(" DPAD DOWN", self.dpad_down.clone()));
        items.push(MenuItem::new(" DPAD LEFT", self.dpad_left.clone()));
        items.push(MenuItem::new(" JOYSTICK X", self.joystick_x.clone()));
        items.push(MenuItem::new(" JOYSTICK Y", self.joystick_y.clone()));
        items.push(MenuItem::new(" ROTARY", self.rotary.clone()));

        items
    }
}
