use super::enums::DiscoverySubmenu;
use super::enums::MainMenu;
use super::structures::InputState;
use super::structures::InputStateEnum;
use super::structures::SelectString;
use super::structures::SubMenuSelect;
use super::theme::DynamicTheme;
use super::utils::discovery_submenu_items;
use super::utils::discovery_to_menu_items;

use crate::devices::DevicesBuilder;

use crate::display::setup_display;
use crate::display::GraphicsDisplay;
use crate::display::SPIMode;

use embsys::crates::defmt;
use embsys::crates::embassy_executor;
use embsys::crates::embassy_futures;
use embsys::crates::embassy_rp;
use embsys::crates::embedded_graphics;
use embsys::exts::std;

use std::string::String;
use std::vec::Vec;

use embassy_executor::SendSpawner;

use embedded_graphics::Drawable;
use embedded_menu::interaction::Action;
use embedded_menu::interaction::Interaction;
use embedded_menu::interaction::Navigation;
use embedded_menu::items::MenuItem;
use embedded_menu::Menu;
use embedded_menu::MenuState;

use archon_core::devices::layout::DeviceLayout;
use archon_core::discovery::AnnounceInformation;
use archon_core::discovery::DiscoveryInformation;
use archon_core::discovery::DiscoveryStatus;
use archon_core::discovery::MultiCastDiscovery;
use archon_core::input::DPad;
use archon_core::input::InputType;

pub async fn display_menu(spawner: SendSpawner) {
    let mut layout: DeviceLayout = DeviceLayout::new();
    DevicesBuilder::build(&mut layout).await;

    let mut display: GraphicsDisplay<SPIMode<'_>> = setup_display();
    main_display_menu(spawner, &mut display, &mut layout).await;
}

pub async fn main_display_menu(
    spawner: SendSpawner,
    display: &mut GraphicsDisplay<SPIMode<'_>>,
    layout: &mut DeviceLayout,
) {
    let mut state: MenuState<_, _, _> = Default::default();

    loop {
        embassy_futures::yield_now().await;
        let inputs: Vec<InputType> = layout.get_inputs().await;

        let mut menu: _ = Menu::with_style("Main Menu", DynamicTheme::style())
            .add_menu_items(MainMenu::to_menu_items())
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
                        let val = menu.interact(Interaction::Action(Action::Select));
                        if let Some(val) = val {
                            match val {
                                MainMenu::Discovery => {
                                    discovery_display_menu(spawner, display, layout).await;
                                }
                                MainMenu::Settings => {}
                                MainMenu::Diagnostics => {
                                    diagnostics_display_menu(display, layout).await;
                                }
                            }
                        }
                    }
                    DPad::Down => {
                        menu.interact(Interaction::Navigation(Navigation::Next));
                    }
                    DPad::Left => {}
                },
                _ => {}
            }
        }
        state = menu.state();
    }
}

pub async fn discovery_display_menu(
    spawner: SendSpawner,
    display: &mut GraphicsDisplay<SPIMode<'_>>,
    layout: &mut DeviceLayout,
) {
    let discovery: MultiCastDiscovery = MultiCastDiscovery::new();
    let _ = discovery.join().await;
    let status: &DiscoveryStatus = discovery.start_discovery(&spawner).await.unwrap();

    let mut state: MenuState<_, _, _> = Default::default();

    loop {
        embassy_futures::yield_now().await;
        let inputs: Vec<InputType> = layout.get_inputs().await;

        let mut discovered: Vec<DiscoveryInformation> = status.discovered();
        discovered.push(DiscoveryInformation::new(
            [192, 168, 0, 132],
            [192, 168, 0, 79],
            AnnounceInformation::new("Some Receiver", 8000),
        )); // Debug Purposes

        let items: Vec<MenuItem<String, Option<usize>, SubMenuSelect, true>> =
            discovery_to_menu_items(&discovered);

        let mut menu: _ = Menu::with_style("Discovery", DynamicTheme::style())
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
                                discovery_display_submenu(display, layout, &discovery, info).await;
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

pub async fn discovery_display_submenu(
    display: &mut GraphicsDisplay<SPIMode<'_>>,
    layout: &mut DeviceLayout,
    discovery: &MultiCastDiscovery,
    info: &DiscoveryInformation,
) {
    let mut state: _ = Default::default();
    let mut theme: _ = DynamicTheme::style();

    loop {
        embassy_futures::yield_now().await;
        let inputs: Vec<InputType> = layout.get_inputs().await;
        let items: Vec<MenuItem<&str, DiscoverySubmenu, SelectString, true>> =
            discovery_submenu_items(info);

        let mut menu: _ = Menu::with_style("Discovery", theme)
            .add_menu_items(items)
            .build_with_state(state);

        theme = DynamicTheme::from_actionable(menu.selected_value().is_actionable());

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
                                DiscoverySubmenu::Connect => {
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

pub async fn diagnostics_display_menu(
    display: &mut GraphicsDisplay<SPIMode<'_>>,
    layout: &mut DeviceLayout,
) {
    let mut state: MenuState<_, _, _> = Default::default();

    loop {
        embassy_futures::yield_now().await;

        let inputs: Vec<InputType> = layout.get_inputs().await;
        let input_state: InputState = match InputState::from_inputs(&inputs).await {
            Ok(state) => state,
            Err(_) => continue,
        };
        let items: Vec<MenuItem<&str, (), InputStateEnum, true>> = input_state.to_menu_items();

        let mut menu: _ = Menu::with_style("Diagnostics", DynamicTheme::hidden())
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
                        menu.interact(Interaction::Action(Action::Select));
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
