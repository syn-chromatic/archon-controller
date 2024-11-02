use super::enums::DiscoverySubmenu;
use super::structures::SelectString;
use super::structures::SubMenuSelect;

use embsys::exts::std;

use std::string::String;
use std::string::ToString;
use std::vec::Vec;

use archon_core::discovery::DiscoveryInformation;
use embedded_menu::items::MenuItem;

pub fn discovery_to_menu_items(
    discovered: &Vec<DiscoveryInformation>,
) -> Vec<MenuItem<String, Option<usize>, SubMenuSelect, true>> {
    let mut items: Vec<_> = Vec::new();

    for (idx, info) in discovered.iter().enumerate() {
        let title_text: String = " ".to_string() + info.announce_info().name();
        let value: SubMenuSelect = SubMenuSelect::new(idx);
        let item: MenuItem<String, Option<usize>, SubMenuSelect, true> =
            MenuItem::new(title_text, value).with_value_converter(|select| select.index());
        items.push(item);
    }

    if items.len() == 0 {
        items.push(
            MenuItem::new(" No Devices..".to_string(), SubMenuSelect::default())
                .with_value_converter(|select| select.index()),
        );
    }

    items
}

pub fn discovery_submenu_items(
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
