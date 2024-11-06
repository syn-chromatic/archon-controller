#![allow(dead_code)]

use super::structures::F32Value;
use super::structures::SelectString;
use super::structures::U16Value;

use embsys::exts::std;
use std::string::ToString;
use std::vec::Vec;

use archon_core::discovery::DiscoveryInformation;
use archon_core::input::DPad;
use archon_core::input::InputType;

use embedded_menu::items::menu_item::SelectValue;
use embedded_menu::items::MenuItem;
use embedded_menu::SelectValue as SelectValueMacro;

#[derive(Copy, Clone, PartialEq, SelectValueMacro)]
pub enum BooleanEnum {
    ON,
    OFF,
}

impl BooleanEnum {
    pub fn new(state: bool) -> Self {
        match state {
            true => BooleanEnum::ON,
            false => BooleanEnum::OFF,
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum ValueEnum {
    F32(F32Value),
    U16(U16Value),
    Boolean(BooleanEnum),
}

impl ValueEnum {
    pub fn u16(value: u16) -> Self {
        ValueEnum::U16(U16Value::new(value))
    }

    pub fn f32(value: f32) -> Self {
        ValueEnum::F32(F32Value::new(value))
    }

    pub fn boolean(value: bool) -> Self {
        ValueEnum::Boolean(BooleanEnum::new(value))
    }
}

impl SelectValue for ValueEnum {
    fn marker(&self) -> &str {
        match self {
            ValueEnum::F32(f32_value) => f32_value.marker(),
            ValueEnum::U16(u16_value) => u16_value.marker(),
            ValueEnum::Boolean(button_enum) => button_enum.marker(),
        }
    }
}

#[derive(Copy, Clone)]
pub enum MainMenu {
    Discovery,
    Settings,
    Diagnostics,
}

impl MainMenu {
    fn as_str(&self) -> &str {
        match self {
            MainMenu::Discovery => "Discovery",
            MainMenu::Settings => "Settings",
            MainMenu::Diagnostics => "Diagnostics",
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

#[derive(Copy, Clone, PartialEq)]
pub enum DiscoverySubmenu {
    Name,
    RemoteIP,
    LocalIP,
    TCPPort,
    Connect,
}

impl DiscoverySubmenu {
    fn name_item(
        value: SelectString,
    ) -> MenuItem<&'static str, DiscoverySubmenu, SelectString, true> {
        let title_text: &str = DiscoverySubmenu::Name.as_str();
        MenuItem::new(title_text, value).with_value_converter(|_| DiscoverySubmenu::Name)
    }

    fn remote_ip_item(
        value: SelectString,
    ) -> MenuItem<&'static str, DiscoverySubmenu, SelectString, true> {
        let title_text: &str = DiscoverySubmenu::RemoteIP.as_str();
        MenuItem::new(title_text, value).with_value_converter(|_| DiscoverySubmenu::RemoteIP)
    }

    fn local_ip_item(
        value: SelectString,
    ) -> MenuItem<&'static str, DiscoverySubmenu, SelectString, true> {
        let title_text: &str = DiscoverySubmenu::LocalIP.as_str();
        MenuItem::new(title_text, value).with_value_converter(|_| DiscoverySubmenu::LocalIP)
    }

    fn tcp_port_item(
        value: SelectString,
    ) -> MenuItem<&'static str, DiscoverySubmenu, SelectString, true> {
        let title_text: &str = DiscoverySubmenu::TCPPort.as_str();
        MenuItem::new(title_text, value).with_value_converter(|_| DiscoverySubmenu::TCPPort)
    }

    fn connect_item() -> MenuItem<&'static str, DiscoverySubmenu, SelectString, true> {
        let title_text: &str = DiscoverySubmenu::Connect.as_str();
        MenuItem::new(title_text, "".into()).with_value_converter(|_| DiscoverySubmenu::Connect)
    }
}

impl DiscoverySubmenu {
    pub fn as_str(&self) -> &str {
        match self {
            DiscoverySubmenu::Name => "Name",
            DiscoverySubmenu::RemoteIP => "ReIP",
            DiscoverySubmenu::LocalIP => "LoIP",
            DiscoverySubmenu::TCPPort => "Port",
            DiscoverySubmenu::Connect => "Connect",
        }
    }

    pub fn to_menu_items(
        info: &DiscoveryInformation,
    ) -> Vec<MenuItem<&str, DiscoverySubmenu, SelectString, true>> {
        let mut items: Vec<MenuItem<&str, DiscoverySubmenu, SelectString, true>> = Vec::new();

        let name: SelectString = info.announce_info().name().into();
        let remote_addr: SelectString = info.remote_addr_string().into();
        let local_addr: SelectString = info.local_addr_string().into();
        let tcp_port: SelectString = info.announce_info().tcp_port().to_string().into();

        items.push(DiscoverySubmenu::name_item(name));
        items.push(DiscoverySubmenu::remote_ip_item(remote_addr));
        items.push(DiscoverySubmenu::local_ip_item(local_addr));
        items.push(DiscoverySubmenu::tcp_port_item(tcp_port));
        items.push(DiscoverySubmenu::connect_item());

        items
    }

    pub fn is_actionable(&self) -> bool {
        match self {
            DiscoverySubmenu::Connect => true,
            _ => false,
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum DiagnosticsMenu {
    DPadUp,
    DPadRight,
    DPadDown,
    DPadLeft,
    JoyStickX,
    JoyStickY,
    Rotary,
}

impl DiagnosticsMenu {
    fn dpad_up_item(value: ValueEnum) -> MenuItem<&'static str, DiagnosticsMenu, ValueEnum, true> {
        let title_text: &str = DiagnosticsMenu::DPadUp.as_str();
        MenuItem::new(title_text, value).with_value_converter(|_| DiagnosticsMenu::DPadUp)
    }

    fn dpad_right_item(
        value: ValueEnum,
    ) -> MenuItem<&'static str, DiagnosticsMenu, ValueEnum, true> {
        let title_text: &str = DiagnosticsMenu::DPadRight.as_str();
        MenuItem::new(title_text, value).with_value_converter(|_| DiagnosticsMenu::DPadRight)
    }

    fn dpad_down_item(
        value: ValueEnum,
    ) -> MenuItem<&'static str, DiagnosticsMenu, ValueEnum, true> {
        let title_text: &str = DiagnosticsMenu::DPadDown.as_str();
        MenuItem::new(title_text, value).with_value_converter(|_| DiagnosticsMenu::DPadDown)
    }

    fn dpad_left_item(
        value: ValueEnum,
    ) -> MenuItem<&'static str, DiagnosticsMenu, ValueEnum, true> {
        let title_text: &str = DiagnosticsMenu::DPadLeft.as_str();
        MenuItem::new(title_text, value).with_value_converter(|_| DiagnosticsMenu::DPadLeft)
    }

    fn joystick_x_item(
        value: ValueEnum,
    ) -> MenuItem<&'static str, DiagnosticsMenu, ValueEnum, true> {
        let title_text: &str = DiagnosticsMenu::JoyStickX.as_str();
        MenuItem::new(title_text, value).with_value_converter(|_| DiagnosticsMenu::JoyStickX)
    }

    fn joystick_y_item(
        value: ValueEnum,
    ) -> MenuItem<&'static str, DiagnosticsMenu, ValueEnum, true> {
        let title_text: &str = DiagnosticsMenu::JoyStickY.as_str();
        MenuItem::new(title_text, value).with_value_converter(|_| DiagnosticsMenu::JoyStickY)
    }

    fn rotary_item(value: ValueEnum) -> MenuItem<&'static str, DiagnosticsMenu, ValueEnum, true> {
        let title_text: &str = DiagnosticsMenu::Rotary.as_str();
        MenuItem::new(title_text, value).with_value_converter(|_| DiagnosticsMenu::Rotary)
    }
}

impl DiagnosticsMenu {
    pub fn as_str(&self) -> &str {
        match self {
            DiagnosticsMenu::DPadUp => "DPad UP",
            DiagnosticsMenu::DPadRight => "DPad Right",
            DiagnosticsMenu::DPadDown => "DPad Down",
            DiagnosticsMenu::DPadLeft => "DPad Left",
            DiagnosticsMenu::JoyStickX => "JoyStick X",
            DiagnosticsMenu::JoyStickY => "JoyStick Y",
            DiagnosticsMenu::Rotary => "Rotary",
        }
    }

    pub fn to_menu_items(
        inputs: &Vec<InputType>,
    ) -> Vec<MenuItem<&'static str, DiagnosticsMenu, ValueEnum, true>> {
        let mut dpad_up: ValueEnum = ValueEnum::boolean(false);
        let mut dpad_right: ValueEnum = ValueEnum::boolean(false);
        let mut dpad_down: ValueEnum = ValueEnum::boolean(false);
        let mut dpad_left: ValueEnum = ValueEnum::boolean(false);
        let mut joystick_x: ValueEnum = ValueEnum::u16(0);
        let mut joystick_y: ValueEnum = ValueEnum::u16(0);
        let mut rotary: ValueEnum = ValueEnum::u16(0);

        for input in inputs {
            match input {
                InputType::DPad(input_dpad) => match input_dpad.dpad() {
                    DPad::Up => dpad_up = ValueEnum::boolean(true),
                    DPad::Right => dpad_right = ValueEnum::boolean(true),
                    DPad::Down => dpad_down = ValueEnum::boolean(true),
                    DPad::Left => dpad_left = ValueEnum::boolean(true),
                },
                InputType::JoyStick(input_joy_stick) => {
                    joystick_x = ValueEnum::u16(input_joy_stick.x());
                    joystick_y = ValueEnum::u16(input_joy_stick.y());
                }
                InputType::ASCII(_input_ascii) => {}
                InputType::Rotary(input_rotary) => {
                    rotary = ValueEnum::u16(input_rotary.value());
                }
                InputType::Button(_input_button) => {}
            }
        }

        let mut items: _ = Vec::new();
        items.push(Self::dpad_up_item(dpad_up));
        items.push(Self::dpad_right_item(dpad_right));
        items.push(Self::dpad_down_item(dpad_down));
        items.push(Self::dpad_left_item(dpad_left));
        items.push(Self::joystick_x_item(joystick_x));
        items.push(Self::joystick_y_item(joystick_y));
        items.push(Self::rotary_item(rotary));

        items
    }
}
