#![allow(dead_code)]

use super::super::enums::MainMenu;
use super::super::indicator::DynShape;
use super::super::style::DynMenuStyle;
use super::super::theme::StandardTheme;
use super::about::about_menu;
use super::diagnostics::diagnostics_menu;
use super::discovery::discovery_menu;
use super::settings::settings_menu;

use crate::devices::DevicesBuilder;

use crate::display::setup_display;
use crate::display::GraphicsDisplay;
use crate::display::SPIMode;

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

use archon_core::devices::layout::DeviceLayout;
use archon_core::input::DPad;
use archon_core::input::InputType;

pub async fn start_interface(spawner: SendSpawner) {
    let mut layout: DeviceLayout = DeviceLayout::new();
    DevicesBuilder::build(&mut layout).await;

    let mut display: GraphicsDisplay<SPIMode<'_>> = setup_display();
    main_menu(spawner, &mut display, &mut layout).await;
}

pub async fn main_menu(
    spawner: SendSpawner,
    display: &mut GraphicsDisplay<SPIMode<'_>>,
    layout: &mut DeviceLayout,
) {
    let style: _ = DynMenuStyle::new(StandardTheme, DynShape::Triangle);
    let mut state: _ = MenuState::default();

    loop {
        embassy_futures::yield_now().await;
        let inputs: Vec<InputType> = layout.get_inputs().await;

        let items: _ = MainMenu::to_menu_items();
        let mut menu: _ = Menu::with_style("Main Menu", *style)
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
                        let val: Option<MainMenu> =
                            menu.interact(Interaction::Action(Action::Select));
                        if let Some(val) = val {
                            match val {
                                MainMenu::Discovery => {
                                    discovery_menu(spawner, display, layout).await;
                                }
                                MainMenu::Settings => {
                                    settings_menu(spawner, display, layout).await;
                                }
                                MainMenu::Diagnostics => {
                                    diagnostics_menu(display, layout).await;
                                }
                                MainMenu::About => {
                                    about_menu(display, layout).await;
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
