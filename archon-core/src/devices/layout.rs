use super::dpad::DPadDevice;
use super::joystick::JoyStickDevice;

use crate::input::InputType;

use embsys::crates::embassy_time;
use embsys::exts::std;

use std::vec::Vec;

use embassy_time::Duration;
use embassy_time::Instant;

pub struct DevicePolling {
    poll_duration: Duration,
    poll_instant: Option<Instant>,
}

impl DevicePolling {
    pub fn new(poll_duration: Duration) -> Self {
        let poll_instant = None;
        Self {
            poll_duration,
            poll_instant,
        }
    }

    pub fn poll(&mut self) -> bool {
        if let Some(instant) = self.poll_instant {
            let duration: Duration = instant.elapsed();
            if duration >= self.poll_duration {
                self.poll_instant = Some(Instant::now());
                return true;
            }
        } else {
            self.poll_instant = Some(Instant::now());
            return true;
        }

        false
    }
}

pub struct DeviceLayout {
    dpad: Option<DPadDevice>,
    joystick: Option<JoyStickDevice>,
}

impl DeviceLayout {
    async fn add_dpad_inputs(&mut self, inputs: &mut Vec<InputType>) {
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
    pub const fn new() -> Self {
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
        self.add_dpad_inputs(&mut inputs).await;
        self.add_joystick_inputs(&mut inputs).await;
        inputs
    }

    pub async fn get_dpad_inputs(&mut self) -> Vec<InputType> {
        let mut inputs: Vec<InputType> = Vec::new();

        if let Some(dpad) = &mut self.dpad {
            for input in dpad.get_inputs_as_types() {
                if let Some(input) = input {
                    inputs.push(input);
                }
            }
        }
        inputs
    }

    pub async fn get_joystick_input(&mut self) -> Vec<InputType> {
        let mut inputs: Vec<InputType> = Vec::new();

        if let Some(joystick) = &mut self.joystick {
            if let Ok(input) = joystick.get_input_as_type().await {
                if let Some(input) = input {
                    inputs.push(input);
                }
            }
        }
        inputs
    }
}
