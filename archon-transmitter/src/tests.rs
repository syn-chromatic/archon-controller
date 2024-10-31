#![allow(dead_code)]
#![allow(unused_imports)]

use embsys::crates::embedded_graphics;
use embsys::exts::std;

use std::cell::Cell;
use std::string::String;
use std::string::ToString;
use std::sync::Arc;
use std::sync::Mutex;
use std::vec::Vec;

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

pub struct InputState {
    dpad_up: ButtonEnum,
    dpad_right: ButtonEnum,
    dpad_down: ButtonEnum,
    dpad_left: ButtonEnum,
    joystick_x: U16Value,
    joystick_y: U16Value,
    rotary: U16Value,
}

impl From<&Vec<InputType>> for InputState {
    fn from(inputs: &Vec<InputType>) -> Self {
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

pub async fn test_display_menu() {
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
    let mut state: MenuState<_, _, _> = Default::default();

    let style: MenuStyle<
        AnimatedTriangle,
        Programmed,
        embedded_menu::selection_indicator::StaticPosition,
        (),
        MenuTheme,
    > = MenuStyle::new(MenuTheme).with_selection_indicator(AnimatedTriangle::new(40));

    loop {
        let inputs: Vec<InputType> = layout.get_inputs().await;
        let input_state: InputState = (&inputs).into();

        let mut menu = Menu::with_style("Menu Title", style)
            .add_item(" DPAD UP", input_state.dpad_up, |_| ())
            .add_item(" DPAD RIGHT", input_state.dpad_right, |_| ())
            .add_item(" DPAD DOWN", input_state.dpad_down, |_| ())
            .add_item(" DPAD LEFT", input_state.dpad_left, |_| ())
            .add_item(" JOYSTICK X", input_state.joystick_x, |_| ())
            .add_item(" JOYSTICK Y", input_state.joystick_y, |_| ())
            .add_item(" ROTARY", input_state.rotary, |_| ())
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
                        menu.interact(Interaction::Action(Action::Return(())));
                    }
                },
                InputType::JoyStick(input_joy_stick) => {}
                InputType::ASCII(input_ascii) => {}
                InputType::Rotary(input_rotary) => {}
                InputType::Button(input_button) => {}
            }
        }
        state = menu.state();
    }
}
