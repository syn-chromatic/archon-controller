#![allow(dead_code)]

use embsys::crates::embedded_graphics;

use embedded_graphics::Drawable;
use embedded_menu::interaction::programmed::Programmed;
use embedded_menu::interaction::Action;
use embedded_menu::interaction::Interaction;
use embedded_menu::interaction::Navigation;
use embedded_menu::selection_indicator::style::AnimatedTriangle;
use embedded_menu::selection_indicator::AnimatedPosition;
use embedded_menu::Menu;
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

    let style: MenuStyle<AnimatedTriangle, Programmed, AnimatedPosition, (), MenuTheme> =
        MenuStyle::new(MenuTheme)
            .with_selection_indicator(AnimatedTriangle::new(40))
            .with_animated_selection_indicator(2);

    let mut menu = Menu::with_style("Menu Title", style)
        .add_item(" Whatever", ">", |_| ())
        .add_item(" Some Setting 1", false, |_| ())
        .add_item(" Some Setting 2", false, |_| ())
        .add_item(" Some Setting 3", TestEnum::A, |_| ())
        .add_item(" Hubba Hubba", false, |_| ())
        .build();

    loop {
        menu.update(display.get());
        menu.draw(display.get()).unwrap();

        display.get().flush();
        display.get().clear(false);

        let inputs = layout.get_inputs().await;
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
    }
}
