#![allow(dead_code)]

use super::super::enums::SettingsMenu;
use super::super::enums::WIFISubmenu;
use super::super::indicator::DynShape;
use super::super::style::DynMenuStyle;
use super::super::theme::StandardTheme;
use super::super::traits::ActionableSelect;

use crate::display::GraphicsDisplay;
use crate::display::SPIMode;

use embsys::crates::embassy_futures;
use embsys::crates::embedded_graphics;
use embsys::exts::std;

use std::vec::Vec;

use embedded_graphics::Drawable;
use embedded_menu::interaction::Action;
use embedded_menu::interaction::Interaction;
use embedded_menu::interaction::Navigation;
use embedded_menu::Menu;
use embedded_menu::MenuState;

use archon_core::devices::layout::DeviceLayout;
use archon_core::input::DPad;
use archon_core::input::InputType;

pub async fn settings_menu(display: &mut GraphicsDisplay<SPIMode<'_>>, layout: &mut DeviceLayout) {
    let style: _ = DynMenuStyle::new(StandardTheme, DynShape::Triangle);
    let mut state: _ = MenuState::default();

    loop {
        embassy_futures::yield_now().await;

        let inputs: Vec<InputType> = layout.get_inputs().await;
        let items: _ = SettingsMenu::to_menu_items();

        let mut menu: _ = Menu::with_style("Settings", *style)
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
                        if let Some(value) = menu.interact(Interaction::Action(Action::Select)) {
                            match value {
                                SettingsMenu::WIFI => {
                                    wifi_submenu(display, layout).await;
                                }
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

pub async fn wifi_submenu(display: &mut GraphicsDisplay<SPIMode<'_>>, layout: &mut DeviceLayout) {
    let mut style: _ = DynMenuStyle::new(StandardTheme, DynShape::Triangle);
    let mut state: _ = MenuState::default();

    loop {
        embassy_futures::yield_now().await;

        let inputs: Vec<InputType> = layout.get_inputs().await;
        let items: _ = WIFISubmenu::to_menu_items();

        let mut menu: _ = Menu::with_style("Wi-Fi", *style)
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
                        if let Some(value) = menu.interact(Interaction::Action(Action::Select)) {
                            match value {
                                WIFISubmenu::Connect => {}
                                WIFISubmenu::Disconnect => {}
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
