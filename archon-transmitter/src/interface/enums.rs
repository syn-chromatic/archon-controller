#![allow(dead_code)]

use super::structures::F32Value;
use super::structures::U16Value;
use super::traits::ActionableSelect;

use embsys::drivers::hardware;
use embsys::exts::std;
use embsys::helpers;

use std::string::String;
use std::string::ToString;
use std::vec::Vec;

use hardware::HWController;
use hardware::WIFIController;
use hardware::WIFIState;
use hardware::WIFIStatus;
use helpers::formatter::size::format_size;

use archon_core::discovery::DiscoveryInformation;
use archon_core::input::DPad;
use archon_core::input::InputType;
use archon_core::utils::addr_bytes_to_string;

use archon_macros::ToItem;
use archon_macros::ValueConverter;

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
    String(String),
    Str(&'static str),
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

    pub fn string(value: &str) -> ValueEnum {
        ValueEnum::String(value.to_string())
    }

    pub fn str(value: &'static str) -> ValueEnum {
        ValueEnum::Str(value)
    }

    pub fn empty() -> ValueEnum {
        ValueEnum::Str("")
    }

    pub fn arrow() -> ValueEnum {
        ValueEnum::Str(">")
    }
}

impl SelectValue for ValueEnum {
    fn marker(&self) -> &str {
        match self {
            ValueEnum::F32(f32) => f32.marker(),
            ValueEnum::U16(u16) => u16.marker(),
            ValueEnum::Boolean(boolean) => boolean.marker(),
            ValueEnum::String(string) => string,
            ValueEnum::Str(str) => *str,
        }
    }
}

impl From<String> for ValueEnum {
    fn from(value: String) -> Self {
        ValueEnum::String(value)
    }
}

impl From<&'static str> for ValueEnum {
    fn from(value: &'static str) -> Self {
        ValueEnum::Str(value)
    }
}

#[derive(Copy, Clone, PartialEq, ValueConverter, ToItem)]
pub enum MainMenu {
    Discovery,
    Settings,
    Diagnostics,
    About,
}

impl MainMenu {
    pub fn as_str(&self) -> &str {
        match self {
            MainMenu::Discovery => "Discovery",
            MainMenu::Settings => "Settings",
            MainMenu::Diagnostics => "Diagnostics",
            MainMenu::About => "About",
        }
    }

    pub fn to_menu_items<'a>() -> Vec<MenuItem<&'a str, Self, ValueEnum, true>> {
        let mut items: _ = Vec::new();

        items.push(MainMenu::Discovery.item(ValueEnum::arrow()));
        items.push(MainMenu::Settings.item(ValueEnum::arrow()));
        items.push(MainMenu::Diagnostics.item(ValueEnum::arrow()));
        items.push(MainMenu::About.item(ValueEnum::arrow()));

        items
    }
}

#[derive(Copy, Clone, PartialEq, ValueConverter, ToItem)]
pub enum DiscoverySubmenu {
    Name,
    RemoteIP,
    LocalIP,
    TCPPort,
    Connect,
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
    ) -> Vec<MenuItem<&str, Self, ValueEnum, true>> {
        let mut items: _ = Vec::new();

        let name: ValueEnum = info.announce_info().name().to_string().into();
        let remote_addr: ValueEnum = info.remote_addr_string().into();
        let local_addr: ValueEnum = info.local_addr_string().into();
        let tcp_port: ValueEnum = info.announce_info().tcp_port().to_string().into();
        let connect: ValueEnum = ValueEnum::empty();

        items.push(DiscoverySubmenu::Name.item(name));
        items.push(DiscoverySubmenu::RemoteIP.item(remote_addr));
        items.push(DiscoverySubmenu::LocalIP.item(local_addr));
        items.push(DiscoverySubmenu::TCPPort.item(tcp_port));
        items.push(DiscoverySubmenu::Connect.item(connect));

        items
    }
}

impl ActionableSelect for DiscoverySubmenu {
    fn is_actionable(&self) -> bool {
        match self {
            DiscoverySubmenu::Connect => true,
            _ => false,
        }
    }
}

#[derive(Copy, Clone, PartialEq, ValueConverter, ToItem)]
pub enum SettingsMenu {
    WIFI,
}

impl SettingsMenu {
    pub fn as_str(&self) -> &str {
        match self {
            SettingsMenu::WIFI => "Wi-Fi",
        }
    }

    pub fn to_menu_items<'a>() -> Vec<MenuItem<&'a str, Self, ValueEnum, true>> {
        let mut items: _ = Vec::new();

        items.push(SettingsMenu::WIFI.item(ValueEnum::arrow()));

        items
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum WIFIConnect {
    Connect,
    Connecting,
    Disconnect,
}

impl WIFIConnect {
    pub fn as_str(&self) -> &str {
        match self {
            WIFIConnect::Connect => "Connect",
            WIFIConnect::Connecting => "Connecting",
            WIFIConnect::Disconnect => "Disconnect",
        }
    }

    pub fn value_converter(&self) -> fn(ValueEnum) -> WIFISubmenu {
        match self {
            WIFIConnect::Connect => |_| WIFISubmenu::Connect(WIFIConnect::Connect),
            WIFIConnect::Connecting => |_| WIFISubmenu::Connect(WIFIConnect::Connecting),
            WIFIConnect::Disconnect => |_| WIFISubmenu::Connect(WIFIConnect::Disconnect),
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum WIFISubmenu {
    SSID,
    Status,
    Address,
    Connect(WIFIConnect),
}

impl WIFISubmenu {
    fn value_converter(&self) -> fn(ValueEnum) -> Self {
        match self {
            WIFISubmenu::SSID => |_| WIFISubmenu::SSID,
            WIFISubmenu::Status => |_| WIFISubmenu::Status,
            WIFISubmenu::Address => |_| WIFISubmenu::Address,
            WIFISubmenu::Connect(connect) => connect.value_converter(),
        }
    }

    fn item(&self, value: ValueEnum) -> MenuItem<&str, Self, ValueEnum, true> {
        let title_text = self.as_str();
        MenuItem::new(title_text, value).with_value_converter(self.value_converter())
    }

    fn get_ssid(state: &WIFIState) -> ValueEnum {
        if let Some(link) = &state.link {
            return ValueEnum::string(&link.ssid);
        }
        ValueEnum::string("")
    }

    fn get_status(state: &WIFIState) -> ValueEnum {
        match state.status {
            WIFIStatus::Idle => ValueEnum::str("Idle"),
            WIFIStatus::JoiningNetwork => ValueEnum::str("Joining"),
            WIFIStatus::JoiningNetworkFailed => ValueEnum::str("JoinFailed"),
            WIFIStatus::ConfiguringDHCP => ValueEnum::str("DHCPSetup"),
            WIFIStatus::ConfiguringDHCPFailed => ValueEnum::str("DHCPFailed"),
            WIFIStatus::ConnectedUnassigned => ValueEnum::str("Connected"),
            WIFIStatus::ConnectedDHCP => ValueEnum::str("Connected"),
            WIFIStatus::ConnectedStatic => ValueEnum::str("Connected"),
        }
    }

    fn get_address() -> ValueEnum {
        if let Some(config) = WIFIController::as_mut().get_config_v4() {
            let addr_bytes: [u8; 4] = config.address.address().octets();
            let addr_string: String = addr_bytes_to_string(addr_bytes);
            return ValueEnum::String(addr_string);
        }

        ValueEnum::str("Unassigned")
    }

    fn get_connection(state: &WIFIState) -> WIFIConnect {
        match state.status {
            WIFIStatus::Idle => WIFIConnect::Connect,
            WIFIStatus::JoiningNetwork => WIFIConnect::Connecting,
            WIFIStatus::JoiningNetworkFailed => WIFIConnect::Connect,
            WIFIStatus::ConfiguringDHCP => WIFIConnect::Connecting,
            WIFIStatus::ConfiguringDHCPFailed => WIFIConnect::Connect,
            WIFIStatus::ConnectedUnassigned => WIFIConnect::Disconnect,
            WIFIStatus::ConnectedDHCP => WIFIConnect::Disconnect,
            WIFIStatus::ConnectedStatic => WIFIConnect::Disconnect,
        }
    }

    fn get_connection_item(connect: WIFIConnect) -> MenuItem<&'static str, Self, ValueEnum, true> {
        match connect {
            WIFIConnect::Connect => {
                WIFISubmenu::Connect(WIFIConnect::Connect).item(ValueEnum::empty())
            }
            WIFIConnect::Connecting => {
                WIFISubmenu::Connect(WIFIConnect::Connecting).item(ValueEnum::empty())
            }
            WIFIConnect::Disconnect => {
                WIFISubmenu::Connect(WIFIConnect::Disconnect).item(ValueEnum::empty())
            }
        }
    }
}

impl WIFISubmenu {
    pub fn as_str(&self) -> &str {
        match self {
            WIFISubmenu::SSID => "SSID",
            WIFISubmenu::Status => "Stat",
            WIFISubmenu::Address => "Addr",
            WIFISubmenu::Connect(connect) => connect.as_str(),
        }
    }

    pub fn to_menu_items<'a>() -> Vec<MenuItem<&'a str, Self, ValueEnum, true>> {
        let state: WIFIState = WIFIController::state();

        let ssid: ValueEnum = Self::get_ssid(&state);
        let status: ValueEnum = Self::get_status(&state);
        let addr: ValueEnum = Self::get_address();
        let connect: WIFIConnect = Self::get_connection(&state);

        let mut items: _ = Vec::new();

        items.push(WIFISubmenu::SSID.item(ssid));
        items.push(WIFISubmenu::Status.item(status));
        items.push(WIFISubmenu::Address.item(addr));
        items.push(Self::get_connection_item(connect));

        items
    }
}

impl ActionableSelect for WIFISubmenu {
    fn is_actionable(&self) -> bool {
        match self {
            WIFISubmenu::Connect(_) => true,
            _ => false,
        }
    }
}

#[derive(Copy, Clone, PartialEq, ValueConverter, ToItem)]
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
    ) -> Vec<MenuItem<&'static str, Self, ValueEnum, true>> {
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
        items.push(Self::DPadUp.item(dpad_up));
        items.push(Self::DPadRight.item(dpad_right));
        items.push(Self::DPadDown.item(dpad_down));
        items.push(Self::DPadLeft.item(dpad_left));
        items.push(Self::JoyStickX.item(joystick_x));
        items.push(Self::JoyStickY.item(joystick_y));
        items.push(Self::Rotary.item(rotary));

        items
    }
}

#[derive(Copy, Clone, PartialEq, ValueConverter, ToItem)]
pub enum AboutMenu {
    CPU,
    Frequency,
    FreeMemory,
    UsedMemory,
    TotalMemory,
    SysVoltage,
}

impl AboutMenu {
    async fn get_sys_voltage() -> ValueEnum {
        // Needed to disable LED to get accurate sys voltage
        // As LED is connected to CYW43 and the chip uses GP29

        let mut sys_voltage: f32 = 0.0;

        if let Ok(voltage) = HWController::sys_voltage().await {
            sys_voltage = voltage;
        }

        ValueEnum::f32(sys_voltage)
    }
}

impl AboutMenu {
    pub fn as_str(&self) -> &str {
        match self {
            AboutMenu::CPU => "CPU",
            AboutMenu::Frequency => "Frequency",
            AboutMenu::FreeMemory => "FreeMem",
            AboutMenu::UsedMemory => "UsedMem",
            AboutMenu::TotalMemory => "TotalMem",
            AboutMenu::SysVoltage => "SysVoltage",
        }
    }

    pub async fn to_menu_items<'a>() -> Vec<MenuItem<&'a str, Self, ValueEnum, true>> {
        let cpu: ValueEnum = ValueEnum::Str("RP2040");
        let frequency: ValueEnum = ValueEnum::Str("125Mhz");
        let free_memory: ValueEnum =
            ValueEnum::String(format_size(crate::ALLOCATOR.get_free_memory(), 1));
        let used_memory: ValueEnum =
            ValueEnum::String(format_size(crate::ALLOCATOR.get_used_memory(), 1));
        let total_memory: ValueEnum =
            ValueEnum::String(format_size(crate::ALLOCATOR.get_total_memory(), 1));
        let sys_voltage: ValueEnum = Self::get_sys_voltage().await;

        let mut items: _ = Vec::new();

        items.push(AboutMenu::CPU.item(cpu));
        items.push(AboutMenu::Frequency.item(frequency));
        items.push(AboutMenu::FreeMemory.item(free_memory));
        items.push(AboutMenu::UsedMemory.item(used_memory));
        items.push(AboutMenu::TotalMemory.item(total_memory));
        items.push(AboutMenu::SysVoltage.item(sys_voltage));

        items
    }
}
