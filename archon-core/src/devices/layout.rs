use super::dpad::DPadDevice;
use super::joystick::JoyStickDevice;

use crate::input::InputType;

use embsys::exts::std;
use std::vec::Vec;

pub struct DeviceLayout {
    dpad: Option<DPadDevice>,
    joystick: Option<JoyStickDevice>,
}

impl DeviceLayout {
    fn add_dpad_inputs(&mut self, inputs: &mut Vec<InputType>) {
        if let Some(dpad) = &mut self.dpad {
            for input in dpad.get_inputs_as_types() {
                if let Some(input) = input {
                    inputs.push(input);
                }
            }
        }
    }

    async fn add_joystick_inputs(&mut self, inputs: &mut Vec<InputType>) {
        if let Some(joystick) = &mut self.joystick {
            if let Ok(input) = joystick.get_input_as_type().await {
                if let Some(input) = input {
                    inputs.push(input);
                }
            }
        }
    }
}

impl DeviceLayout {
    pub fn new() -> Self {
        let dpad: Option<DPadDevice> = None;
        let joystick: Option<JoyStickDevice> = None;
        Self { dpad, joystick }
    }

    pub fn add_dpad(&mut self, dpad: DPadDevice) {
        self.dpad = Some(dpad);
    }

    pub fn add_joystick(&mut self, joystick: JoyStickDevice) {
        self.joystick = Some(joystick);
    }

    pub async fn get_inputs(&mut self) -> Vec<InputType> {
        let mut inputs: Vec<InputType> = Vec::new();
        self.add_dpad_inputs(&mut inputs);
        self.add_joystick_inputs(&mut inputs).await;
        inputs
    }
}
