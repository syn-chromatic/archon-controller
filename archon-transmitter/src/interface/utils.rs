use super::select::SubMenuSelect;

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
        let title_text: String = info.announce_info().name().to_string();
        let value: SubMenuSelect = SubMenuSelect::new(idx);
        let item: MenuItem<String, Option<usize>, SubMenuSelect, true> =
            MenuItem::new(title_text, value).with_value_converter(|select| select.index());
        items.push(item);
    }

    if items.len() == 0 {
        items.push(
            MenuItem::new("No Devices..".to_string(), SubMenuSelect::default())
                .with_value_converter(|select| select.index()),
        );
    }

    items
}
