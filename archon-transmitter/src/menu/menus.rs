use super::structures::InputState;
use super::structures::MainMenu;

use embsys::crates::embassy_executor;
use embsys::crates::embassy_futures;
use embsys::crates::embedded_graphics;
use embsys::exts::std;

use std::string::ToString;
use std::vec::Vec;

use embassy_executor::SendSpawner;

use embedded_graphics::Drawable;
use embedded_menu::interaction::programmed::Programmed;
use embedded_menu::interaction::Action;
use embedded_menu::interaction::Interaction;
use embedded_menu::interaction::Navigation;
use embedded_menu::items::MenuItem;
use embedded_menu::selection_indicator::style::AnimatedTriangle;
use embedded_menu::selection_indicator::StaticPosition;
use embedded_menu::Menu;
use embedded_menu::MenuState;
use embedded_menu::MenuStyle;

use archon_core::devices::button::ButtonDevice;
use archon_core::devices::dpad::DPadDevice;
use archon_core::devices::joystick::JoyStickDevice;
use archon_core::devices::layout::DeviceLayout;
use archon_core::devices::rotary::RotaryDevice;
use archon_core::discovery::DiscoveryInformation;
use archon_core::discovery::DiscoveryStatus;
use archon_core::discovery::MultiCastDiscovery;
use archon_core::input::DPad;
use archon_core::input::InputType;

use crate::devices::create_dpad_device;
use crate::devices::create_joystick_button_device;
use crate::devices::create_joystick_device;
use crate::devices::create_l1_button_device;
use crate::devices::create_rotary_device;

use crate::display::setup_display;
use crate::display::theme::MenuTheme;
use crate::display::GraphicsDisplay;
use crate::display::SPIMode;

pub async fn display_menu(spawner: SendSpawner) {
    let mut layout: DeviceLayout = DeviceLayout::new();

    let dpad_device: DPadDevice = create_dpad_device();
    let joystick_device: JoyStickDevice = create_joystick_device().await;
    let joystick_button_device: ButtonDevice = create_joystick_button_device();
    let rotary_device: RotaryDevice = create_rotary_device().await;
    let l1_button_device: ButtonDevice = create_l1_button_device();

    layout.add_dpad(dpad_device);
    layout.add_joystick(joystick_device);
    layout.add_button(joystick_button_device);
    layout.add_rotary(rotary_device);
    layout.add_button(l1_button_device);

    let mut display: GraphicsDisplay<SPIMode<'_>> = setup_display();

    main_display_menu(spawner, &mut display, &mut layout).await;
}

pub async fn main_display_menu(
    spawner: SendSpawner,
    display: &mut GraphicsDisplay<SPIMode<'_>>,
    layout: &mut DeviceLayout,
) {
    let mut state: MenuState<_, _, _> = Default::default();
    let style: MenuStyle<AnimatedTriangle, Programmed, StaticPosition, _, MenuTheme> =
        MenuStyle::new(MenuTheme).with_selection_indicator(AnimatedTriangle::new(40));

    loop {
        let inputs: Vec<InputType> = layout.get_inputs().await;

        let mut menu = Menu::with_style("Main Menu", style)
            .add_item(" Discovery", ">", |_| MainMenu::Discovery)
            .add_item(" Settings", ">", |_| MainMenu::Settings)
            .add_item(" Diagnostics", ">", |_| MainMenu::Diagnostics)
            .build_with_state(state);

        menu.update(display.get());
        menu.draw(display.get()).unwrap();

        display.get().flush();
        display.get().clear(false);

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
    let style: MenuStyle<AnimatedTriangle, Programmed, StaticPosition, _, MenuTheme> =
        MenuStyle::new(MenuTheme).with_selection_indicator(AnimatedTriangle::new(40));

    loop {
        embassy_futures::yield_now().await;
        let inputs: Vec<InputType> = layout.get_inputs().await;

        let discovered: Vec<DiscoveryInformation> = status.discovered();

        let mut items: Vec<_> = (0..discovered.len())
            .map(|i| {
                MenuItem::new(
                    " ".to_string() + discovered.get(i).unwrap().announce_info().name(),
                    ">",
                )
            })
            .collect();

        if items.len() == 0 {
            items.push(MenuItem::new(" No Devices..".to_string(), ""));
        }

        let mut menu = Menu::with_style("Discovery", style)
            .add_menu_items(items)
            .build_with_state(state);

        menu.update(display.get());
        menu.draw(display.get()).unwrap();

        display.get().flush();
        display.get().clear(false);

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

pub async fn diagnostics_display_menu(
    display: &mut GraphicsDisplay<SPIMode<'_>>,
    layout: &mut DeviceLayout,
) {
    let mut state: MenuState<_, _, _> = Default::default();
    let style: MenuStyle<AnimatedTriangle, Programmed, StaticPosition, _, MenuTheme> =
        MenuStyle::new(MenuTheme).with_selection_indicator(AnimatedTriangle::new(40));

    loop {
        let inputs: Vec<InputType> = layout.get_inputs().await;
        let input_state: InputState = InputState::from_inputs(&inputs).await;

        let mut menu = Menu::with_style("Diagnostics", style)
            .add_item(" SYS VOLTAGE", input_state.sys_voltage, |_| 0)
            .add_item(" DPAD UP", input_state.dpad_up, |_| 1)
            .add_item(" DPAD RIGHT", input_state.dpad_right, |_| 2)
            .add_item(" DPAD DOWN", input_state.dpad_down, |_| 3)
            .add_item(" DPAD LEFT", input_state.dpad_left, |_| 4)
            .add_item(" JOYSTICK X", input_state.joystick_x, |_| 5)
            .add_item(" JOYSTICK Y", input_state.joystick_y, |_| 6)
            .add_item(" ROTARY", input_state.rotary, |_| 7)
            .build_with_state(state);

        menu.update(display.get());
        menu.draw(display.get()).unwrap();

        display.get().flush();
        display.get().clear(false);

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
