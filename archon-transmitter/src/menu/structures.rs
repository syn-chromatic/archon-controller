use embsys::drivers::hardware::HWController;
use embsys::drivers::hardware::WIFIController;
use embsys::exts::std;

use std::format;
use std::string::String;
use std::string::ToString;
use std::vec::Vec;

use embedded_menu::items::menu_item::SelectValue;
use embedded_menu::items::MenuItem;
use embedded_menu::SelectValue as SelectValueMacro;

use archon_core::input::DPad;
use archon_core::input::InputType;

#[derive(Copy, Clone)]
pub enum MainMenu {
    Discovery,
    Settings,
    Diagnostics,
}

impl MainMenu {
    fn as_str(&self) -> &'static str {
        match self {
            MainMenu::Discovery => " Discovery",
            MainMenu::Settings => " Settings",
            MainMenu::Diagnostics => " Diagnostics",
        }
    }

    fn discovery_item() -> MenuItem<&'static str, Self, &'static str, true> {
        let title_text: &str = MainMenu::Discovery.as_str();
        let value: &str = ">";
        MenuItem::new(title_text, value).with_value_converter(|_| MainMenu::Discovery)
    }

    fn settings_item() -> MenuItem<&'static str, Self, &'static str, true> {
        let title_text: &str = MainMenu::Settings.as_str();
        let value: &str = ">";
        MenuItem::new(title_text, value).with_value_converter(|_| MainMenu::Settings)
    }

    fn diagnostics_item() -> MenuItem<&'static str, Self, &'static str, true> {
        let title_text: &str = MainMenu::Diagnostics.as_str();
        let value: &str = ">";
        MenuItem::new(title_text, value).with_value_converter(|_| MainMenu::Diagnostics)
    }
}

impl MainMenu {
    pub fn to_menu_items() -> Vec<MenuItem<&'static str, Self, &'static str, true>> {
        let mut items: Vec<_> = Vec::new();

        items.push(MainMenu::discovery_item());
        items.push(MainMenu::settings_item());
        items.push(MainMenu::diagnostics_item());

        items
    }
}

#[derive(Copy, Clone, PartialEq, SelectValueMacro)]
pub enum ButtonEnum {
    ON,
    OFF,
}

impl ButtonEnum {
    pub fn new(state: bool) -> Self {
        match state {
            true => ButtonEnum::ON,
            false => ButtonEnum::OFF,
        }
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
    async fn get_sys_voltage() -> InputStateEnum {
        WIFIController::control_mut().gpio_set(0, false).await;
        let sys_voltage: f32 = HWController::sys_voltage_blocking().unwrap();
        InputStateEnum::f32(sys_voltage)
    }
}

impl InputState {
    pub async fn from_inputs(inputs: &Vec<InputType>) -> Self {
        let sys_voltage: InputStateEnum = Self::get_sys_voltage().await;
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
