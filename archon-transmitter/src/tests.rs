#![allow(dead_code)]
#![allow(unused_imports)]

use embsys::crates::defmt;
use embsys::crates::embassy_executor;
use embsys::crates::embassy_futures;
use embsys::crates::embedded_graphics;
use embsys::drivers::hardware::HWController;
use embsys::drivers::hardware::WIFIController;
use embsys::exts::std;

use std::cell::Cell;
use std::format;
use std::string::String;
use std::string::ToString;
use std::sync::Arc;
use std::sync::Mutex;
use std::vec::Vec;

use embassy_executor::SendSpawner;
use embassy_executor::Spawner;

use embedded_graphics::Drawable;
use embedded_menu::interaction::programmed::Programmed;
use embedded_menu::interaction::programmed::ProgrammedAdapter;
use embedded_menu::interaction::Action;
use embedded_menu::interaction::Interaction;
use embedded_menu::interaction::Navigation;
use embedded_menu::items::menu_item::SelectValue;
use embedded_menu::items::MenuItem;
use embedded_menu::items::MenuLine;
use embedded_menu::items::MenuListItem;
use embedded_menu::selection_indicator::style::AnimatedTriangle;
use embedded_menu::selection_indicator::AnimatedPosition;
use embedded_menu::selection_indicator::StaticPosition;
use embedded_menu::Menu;
use embedded_menu::MenuState;
use embedded_menu::MenuStyle;
use embedded_menu::SelectValue;

use archon_core::devices::button::ButtonDevice;
use archon_core::devices::dpad::DPadDevice;
use archon_core::devices::joystick::JoyStickDevice;
use archon_core::devices::layout::DeviceLayout;
use archon_core::devices::rotary::RotaryDevice;
use archon_core::discovery::DiscoveryInformation;
use archon_core::discovery::DiscoveryStatus;
use archon_core::discovery::EstablishInformation;
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

pub async fn test_device_layout() {
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

    loop {
        for input in layout.get_inputs().await {
            input.defmt();
        }
    }
}

#[derive(Copy, Clone, PartialEq, SelectValue)]
pub enum TestEnum {
    A,
    B,
    C,
}

#[derive(Copy, Clone, PartialEq, SelectValue)]
pub enum ButtonEnum {
    ON,
    OFF,
}

#[derive(Clone, PartialEq)]
pub struct U16Value {
    value: u16,
    value_str: String,
}

impl U16Value {
    pub fn new(value: u16) -> Self {
        Self {
            value,
            value_str: value.to_string(),
        }
    }
}

impl From<u16> for U16Value {
    fn from(value: u16) -> Self {
        U16Value::new(value)
    }
}

impl SelectValue for U16Value {
    fn marker(&self) -> &str {
        &self.value_str
    }
}

#[derive(Clone, PartialEq)]
pub struct F32Value {
    value: f32,
    value_str: String,
}

impl F32Value {
    pub fn new(value: f32) -> Self {
        Self {
            value,
            value_str: format!("{:.2}", value),
        }
    }
}

impl From<f32> for F32Value {
    fn from(value: f32) -> Self {
        F32Value::new(value)
    }
}

impl SelectValue for F32Value {
    fn marker(&self) -> &str {
        &self.value_str
    }
}

pub struct InputState {
    sys_voltage: F32Value,
    dpad_up: ButtonEnum,
    dpad_right: ButtonEnum,
    dpad_down: ButtonEnum,
    dpad_left: ButtonEnum,
    joystick_x: U16Value,
    joystick_y: U16Value,
    rotary: U16Value,
}

impl InputState {
    async fn get_sys_voltage() -> F32Value {
        WIFIController::control_mut().gpio_set(0, false).await;
        let sys_voltage: f32 = HWController::sys_voltage_blocking().unwrap();
        F32Value::new(sys_voltage)
    }

    async fn from_inputs(inputs: &Vec<InputType>) -> Self {
        let sys_voltage: F32Value = Self::get_sys_voltage().await;
        let mut dpad_up: ButtonEnum = ButtonEnum::OFF;
        let mut dpad_right: ButtonEnum = ButtonEnum::OFF;
        let mut dpad_down: ButtonEnum = ButtonEnum::OFF;
        let mut dpad_left: ButtonEnum = ButtonEnum::OFF;
        let mut joystick_x: U16Value = U16Value::new(0);
        let mut joystick_y: U16Value = U16Value::new(0);
        let mut rotary: U16Value = U16Value::new(0);

        for input in inputs {
            match input {
                InputType::DPad(input_dpad) => match input_dpad.dpad() {
                    DPad::Up => dpad_up = ButtonEnum::ON,
                    DPad::Right => dpad_right = ButtonEnum::ON,
                    DPad::Down => dpad_down = ButtonEnum::ON,
                    DPad::Left => dpad_left = ButtonEnum::ON,
                },
                InputType::JoyStick(input_joy_stick) => {
                    joystick_x = input_joy_stick.x().into();
                    joystick_y = input_joy_stick.y().into();
                }
                InputType::ASCII(_input_ascii) => {}
                InputType::Rotary(input_rotary) => {
                    rotary = input_rotary.value().into();
                }
                InputType::Button(_input_button) => {}
            }
        }

        InputState {
            sys_voltage,
            dpad_up,
            dpad_right,
            dpad_down,
            dpad_left,
            joystick_x,
            joystick_y,
            rotary,
        }
    }
}

pub async fn test_display_menu(spawner: SendSpawner) {
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

#[derive(Copy, Clone)]
pub enum MainMenu {
    Discovery,
    Settings,
    Diagnostics,
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
                                    diag_display_menu(display, layout).await;
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

pub async fn diag_display_menu(
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
