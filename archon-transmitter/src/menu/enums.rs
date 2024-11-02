use super::structures::SelectString;

use embsys::exts::std;
use std::vec::Vec;

use embedded_menu::items::MenuItem;
use embedded_menu::SelectValue as SelectValueMacro;

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

#[derive(Copy, Clone)]
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
            DiscoverySubmenu::Name => " Name",
            DiscoverySubmenu::RemoteIP => " ReIP",
            DiscoverySubmenu::LocalIP => " LoIP",
            DiscoverySubmenu::TCPPort => " Port",
            DiscoverySubmenu::Connect => " Connect",
        }
    }

    pub fn name_item(
        value: SelectString,
    ) -> MenuItem<&'static str, DiscoverySubmenu, SelectString, true> {
        let title_text: &str = DiscoverySubmenu::Name.as_str();
        MenuItem::new(title_text, value).with_value_converter(|_| DiscoverySubmenu::Name)
    }

    pub fn remote_ip_item(
        value: SelectString,
    ) -> MenuItem<&'static str, DiscoverySubmenu, SelectString, true> {
        let title_text: &str = DiscoverySubmenu::RemoteIP.as_str();
        MenuItem::new(title_text, value).with_value_converter(|_| DiscoverySubmenu::RemoteIP)
    }

    pub fn local_ip_item(
        value: SelectString,
    ) -> MenuItem<&'static str, DiscoverySubmenu, SelectString, true> {
        let title_text: &str = DiscoverySubmenu::LocalIP.as_str();
        MenuItem::new(title_text, value).with_value_converter(|_| DiscoverySubmenu::LocalIP)
    }

    pub fn tcp_port_item(
        value: SelectString,
    ) -> MenuItem<&'static str, DiscoverySubmenu, SelectString, true> {
        let title_text: &str = DiscoverySubmenu::TCPPort.as_str();
        MenuItem::new(title_text, value).with_value_converter(|_| DiscoverySubmenu::TCPPort)
    }

    pub fn connect_item() -> MenuItem<&'static str, DiscoverySubmenu, SelectString, true> {
        let title_text: &str = DiscoverySubmenu::Connect.as_str();
        MenuItem::new(title_text, "".into()).with_value_converter(|_| DiscoverySubmenu::Connect)
    }
}
