use super::button::ButtonDevice;
use super::dpad::DPadDevice;
use super::joystick::JoyStickDevice;
use super::rotary::RotaryDevice;

use crate::input::InputType;

use embsys::exts::std;
use std::vec::Vec;

pub struct DeviceLayout {
    dpad: Vec<DPadDevice>,
    joystick: Vec<JoyStickDevice>,
    rotary: Vec<RotaryDevice>,
    button: Vec<ButtonDevice>,
}

impl DeviceLayout {
    async fn collect_dpad_inputs(&mut self, inputs: &mut Vec<InputType>) {
        for dpad in &mut self.dpad {
            for input in dpad.get_inputs_as_types() {
                if let Some(input) = input {
                    inputs.push(input);
                }
            }
        }
    }

    async fn collect_joystick_inputs(&mut self, inputs: &mut Vec<InputType>) {
        for joystick in &mut self.joystick {
            if let Ok(input) = joystick.get_input_as_type().await {
                if let Some(input) = input {
                    inputs.push(input);
                }
            }
        }
    }

    async fn collect_rotary_inputs(&mut self, inputs: &mut Vec<InputType>) {
        for rotary in &mut self.rotary {
            if let Ok(input) = rotary.get_input_as_type().await {
                if let Some(input) = input {
                    inputs.push(input);
                }
            }
        }
    }

    async fn collect_button_inputs(&mut self, inputs: &mut Vec<InputType>) {
        for button in &mut self.button {
            if let Some(input) = button.get_input_as_type() {
                inputs.push(input);
            }
        }
    }
}

impl DeviceLayout {
    pub const fn new() -> Self {
        Self {
            dpad: Vec::new(),
            joystick: Vec::new(),
            rotary: Vec::new(),
            button: Vec::new(),
        }
    }

    pub fn add_dpad(&mut self, dpad: DPadDevice) {
        self.dpad.push(dpad);
    }

    pub fn add_joystick(&mut self, joystick: JoyStickDevice) {
        self.joystick.push(joystick);
    }

    pub fn add_rotary(&mut self, rotary: RotaryDevice) {
        self.rotary.push(rotary);
    }

    pub fn add_button(&mut self, button: ButtonDevice) {
        self.button.push(button);
    }

    pub async fn get_inputs(&mut self) -> Vec<InputType> {
        let mut inputs: Vec<InputType> = Vec::new();
        self.collect_dpad_inputs(&mut inputs).await;
        self.collect_joystick_inputs(&mut inputs).await;
        self.collect_rotary_inputs(&mut inputs).await;
        self.collect_button_inputs(&mut inputs).await;
        inputs
    }
}
