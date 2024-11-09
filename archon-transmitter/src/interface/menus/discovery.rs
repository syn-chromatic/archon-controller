#![allow(dead_code)]

use super::super::enums::DiscoverySubmenu;
use super::super::indicator::DynShape;
use super::super::style::DynMenuStyle;
use super::super::theme::StandardTheme;
use super::super::traits::ActionableSelect;
use super::super::utils::discovery_to_menu_items;

use crate::device::BufferedDeviceLayout;
use crate::display::GraphicsDisplay;
use crate::display::SPIMode;

use embsys::crates::defmt;
use embsys::crates::embassy_executor;
use embsys::crates::embassy_futures;
use embsys::crates::embedded_graphics;
use embsys::exts::std;

use std::vec::Vec;

use embassy_executor::SendSpawner;

use embedded_graphics::Drawable;
use embedded_menu::interaction::Action;
use embedded_menu::interaction::Interaction;
use embedded_menu::interaction::Navigation;
use embedded_menu::Menu;
use embedded_menu::MenuState;

use archon_core::discovery::AnnounceInformation;
use archon_core::discovery::DiscoveryInformation;
use archon_core::discovery::DiscoveryStatus;
use archon_core::discovery::MultiCastDiscovery;
use archon_core::input::DPad;
use archon_core::input::InputType;

pub async fn discovery_menu(spawner: SendSpawner, display: &mut GraphicsDisplay<SPIMode<'_>>) {
    let discovery: MultiCastDiscovery = MultiCastDiscovery::new();
    let _ = discovery.join().await;
    let status: &DiscoveryStatus = discovery.start_discovery(&spawner).await.unwrap();

    let style: _ = DynMenuStyle::new(StandardTheme, DynShape::Triangle);
    let mut state: _ = MenuState::default();

    loop {
        embassy_futures::yield_now().await;

        let inputs: Vec<InputType> = BufferedDeviceLayout::take_inputs().await;

        let mut discovered: Vec<DiscoveryInformation> = status.discovered();
        discovered.push(DiscoveryInformation::new(
            [192, 168, 0, 132],
            [192, 168, 0, 79],
            AnnounceInformation::new("Some Receiver", 8000),
        )); // Debug Purposes

        let items: _ = discovery_to_menu_items(&discovered);

        let mut menu: _ = Menu::with_style("Discovery", *style)
            .add_menu_items(items)
            .build_with_state(state);

        menu.update(display.get());
        menu.draw(display.get()).unwrap();

        display.refresh();

        for input in inputs {
            match input {
                InputType::DPad(input_dpad) => match input_dpad.dpad() {
                    DPad::Up => {
                        menu.interact(Interaction::Navigation(Navigation::Previous));
                    }
                    DPad::Right => {
                        let index: Option<Option<usize>> =
                            menu.interact(Interaction::Action(Action::Select));
                        if let Some(Some(index)) = index {
                            let info: Option<&DiscoveryInformation> = discovered.get(index);
                            if let Some(info) = info {
                                discovery_submenu(display, &discovery, info).await;
                            }
                        }
                    }
                    DPad::Down => {
                        menu.interact(Interaction::Navigation(Navigation::Next));
                    }
                    DPad::Left => {
                        discovery.stop_discovery().await;
                        return;
                    }
                },
                _ => {}
            }
        }

        state = menu.state();
    }
}

pub async fn discovery_submenu(
    display: &mut GraphicsDisplay<SPIMode<'_>>,
    discovery: &MultiCastDiscovery,
    info: &DiscoveryInformation,
) {
    let mut style: _ = DynMenuStyle::new(StandardTheme, DynShape::Triangle);
    let mut state: _ = MenuState::default();

    loop {
        embassy_futures::yield_now().await;

        let inputs: Vec<InputType> = BufferedDeviceLayout::take_inputs().await;
        let items: _ = DiscoverySubmenu::to_menu_items(info);

        let mut menu: _ = Menu::with_style("Discovery", *style)
            .add_menu_items(items)
            .build_with_state(state);

        menu.selected_value().set_indicator(&mut style);
        menu.update(display.get());
        menu.draw(display.get()).unwrap();

        display.refresh();

        for input in inputs {
            match input {
                InputType::DPad(input_dpad) => match input_dpad.dpad() {
                    DPad::Up => {
                        menu.interact(Interaction::Navigation(Navigation::Previous));
                    }
                    DPad::Right => {
                        let value = menu.interact(Interaction::Action(Action::Select));
                        if let Some(value) = value {
                            match value {
                                DiscoverySubmenu::Connect(_) => {
                                    let result = discovery.connect(info).await;
                                    defmt::info!("Discovery Connection: {:?}", result);
                                }
                                _ => {}
                            }
                        }
                    }
                    DPad::Down => {
                        menu.interact(Interaction::Navigation(Navigation::Next));
                    }
                    DPad::Left => {
                        return;
                    }
                },
                _ => {}
            }
        }

        state = menu.state();
    }
}
