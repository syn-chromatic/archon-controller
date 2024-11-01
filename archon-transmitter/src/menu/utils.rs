use embsys::exts::std;

use std::string::String;
use std::string::ToString;
use std::vec::Vec;

use archon_core::discovery::DiscoveryInformation;
use embedded_menu::items::MenuItem;

pub fn discovery_to_menu_items(
    discovered: &Vec<DiscoveryInformation>,
) -> Vec<MenuItem<String, (), &str, true>> {
    let mut items: Vec<_> = Vec::new();

    for info in discovered.iter() {
        let title_text: String = " ".to_string() + info.announce_info().name();
        let value: &str = ">";
        let item: MenuItem<String, (), &str, true> = MenuItem::new(title_text, value);
        items.push(item);
    }

    if items.len() == 0 {
        items.push(MenuItem::new(" No Devices..".to_string(), ""));
    }

    items
}
