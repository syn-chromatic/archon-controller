#![allow(dead_code)]

use super::select::ValueEnum;
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

use embedded_menu::items::MenuItem;

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

#[derive(Copy, Clone, PartialEq)]
pub enum DiscoveryConnect {
    Connect,
    Connecting,
    Disconnect,
}

impl DiscoveryConnect {
    fn value_converter(&self) -> fn(ValueEnum) -> DiscoverySubmenu {
        match self {
            Self::Connect => |_| DiscoverySubmenu::Connect(Self::Connect),
            Self::Connecting => |_| DiscoverySubmenu::Connect(Self::Connecting),
            Self::Disconnect => |_| DiscoverySubmenu::Connect(Self::Disconnect),
        }
    }
}

impl DiscoveryConnect {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Connect => "Connect",
            Self::Connecting => "Connecting",
            Self::Disconnect => "Disconnect",
        }
    }
}

#[derive(Copy, Clone, PartialEq, ValueConverter, ToItem)]
pub enum DiscoverySubmenu {
    Name,
    RemoteIP,
    LocalIP,
    TCPPort,
    Connect(DiscoveryConnect),
}

impl DiscoverySubmenu {
    fn get_connection() -> DiscoveryConnect {
        DiscoveryConnect::Connect
    }

    fn get_connection_item(
        connect: DiscoveryConnect,
    ) -> MenuItem<&'static str, Self, ValueEnum, true> {
        match connect {
            DiscoveryConnect::Connect => {
                Self::Connect(DiscoveryConnect::Connect).item(ValueEnum::empty())
            }
            DiscoveryConnect::Connecting => {
                Self::Connect(DiscoveryConnect::Connecting).item(ValueEnum::empty())
            }
            DiscoveryConnect::Disconnect => {
                Self::Connect(DiscoveryConnect::Disconnect).item(ValueEnum::empty())
            }
        }
    }
}

impl DiscoverySubmenu {
    pub fn as_str(&self) -> &str {
        match self {
            DiscoverySubmenu::Name => "Name",
            DiscoverySubmenu::RemoteIP => "ReIP",
            DiscoverySubmenu::LocalIP => "LoIP",
            DiscoverySubmenu::TCPPort => "Port",
            DiscoverySubmenu::Connect(connect) => connect.as_str(),
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
        let connect: DiscoveryConnect = Self::get_connection();

        items.push(Self::Name.item(name));
        items.push(Self::RemoteIP.item(remote_addr));
        items.push(Self::LocalIP.item(local_addr));
        items.push(Self::TCPPort.item(tcp_port));
        items.push(Self::get_connection_item(connect));

        items
    }
}

impl ActionableSelect for DiscoverySubmenu {
    fn is_actionable(&self) -> bool {
        match self {
            DiscoverySubmenu::Connect(_) => true,
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
    fn value_converter(&self) -> fn(ValueEnum) -> WIFISubmenu {
        match self {
            Self::Connect => |_| WIFISubmenu::Connect(Self::Connect),
            Self::Connecting => |_| WIFISubmenu::Connect(Self::Connecting),
            Self::Disconnect => |_| WIFISubmenu::Connect(Self::Disconnect),
        }
    }
}

impl WIFIConnect {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Connect => "Connect",
            Self::Connecting => "Connecting",
            Self::Disconnect => "Disconnect",
        }
    }
}

#[derive(Copy, Clone, PartialEq, ValueConverter, ToItem)]
pub enum WIFISubmenu {
    SSID,
    Status,
    Address,
    Connect(WIFIConnect),
}

impl WIFISubmenu {
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
            WIFIConnect::Connect => Self::Connect(WIFIConnect::Connect).item(ValueEnum::empty()),
            WIFIConnect::Connecting => {
                Self::Connect(WIFIConnect::Connecting).item(ValueEnum::empty())
            }
            WIFIConnect::Disconnect => {
                Self::Connect(WIFIConnect::Disconnect).item(ValueEnum::empty())
            }
        }
    }
}

impl WIFISubmenu {
    pub fn as_str(&self) -> &str {
        match self {
            Self::SSID => "SSID",
            Self::Status => "Stat",
            Self::Address => "Addr",
            Self::Connect(connect) => connect.as_str(),
        }
    }

    pub fn to_menu_items<'a>() -> Vec<MenuItem<&'a str, Self, ValueEnum, true>> {
        let state: WIFIState = WIFIController::state();

        let ssid: ValueEnum = Self::get_ssid(&state);
        let status: ValueEnum = Self::get_status(&state);
        let addr: ValueEnum = Self::get_address();
        let connect: WIFIConnect = Self::get_connection(&state);

        let mut items: _ = Vec::new();

        items.push(Self::SSID.item(ssid));
        items.push(Self::Status.item(status));
        items.push(Self::Address.item(addr));
        items.push(Self::get_connection_item(connect));

        items
    }
}

impl ActionableSelect for WIFISubmenu {
    fn is_actionable(&self) -> bool {
        match self {
            Self::Connect(_) => true,
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

        if let Ok(voltage) = HWController::sys_voltage().await {
            return ValueEnum::f32_op(Some(voltage));
        }

        ValueEnum::f32_op(None)
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
