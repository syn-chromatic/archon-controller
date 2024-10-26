#![allow(dead_code)]
#![allow(unused_variables)]

use crate::consts::UDP_BUFFER;
use crate::utils::split_u16;
use crate::utils::u8_to_bool;

use embsys::crates::defmt;

// DATA REPRESENTATION
// [1-byte ID, 2-byte Type ID, X-byte Input Data]

#[repr(u16)]
pub enum InputType {
    /// 6-byte — [1-byte DIRECTION] — [1-byte STATE] — [4-byte PRESS DURATION]
    DPad(InputDPad) = 0,
    /// 4-byte — [2-byte XAXIS, 2-byte YAXIS]
    JoyStick(InputJoyStick) = 1,
    /// 1-byte — [1-byte ASCII]
    ASCII(InputASCII) = 2,
    /// 2-byte — [2-byte ADC]
    Rotary(InputRotary) = 3,
    /// 5-byte — [1-byte STATE] — [4-byte PRESS DURATION]
    Button(InputButton) = 4,
}

impl InputType {
    pub fn from_buffer(buffer: &[u8; UDP_BUFFER]) -> InputType {
        let bytes: &[u8] = &buffer[1..=2];
        let bytes: [u8; 2] = bytes.try_into().unwrap();
        let type_id: u16 = u16::from_be_bytes(bytes);

        match type_id {
            0 => InputType::DPad(InputDPad::from_buffer(buffer)),
            1 => InputType::JoyStick(InputJoyStick::from_buffer(buffer)),
            2 => InputType::ASCII(InputASCII::from_buffer(buffer)),
            3 => InputType::Rotary(InputRotary::from_buffer(buffer)),
            4 => InputType::Button(InputButton::from_buffer(buffer)),
            _ => panic!("Unsupported InputType: {}", { type_id }),
        }
    }

    pub fn defmt(&self) {
        match &self {
            InputType::DPad(dpad) => {
                let id: u8 = dpad.id();
                let name: &str = dpad.dpad().as_str();
                let state: &ButtonState = dpad.state();
                defmt::info!(
                    "DPAD -> ID: {:?} | DIR: {:?} | STATE: {} | DURATION: {}",
                    id,
                    name,
                    state.pressed(),
                    state.duration()
                );
            }
            InputType::JoyStick(joystick) => {
                let id: u8 = joystick.id();
                let xy: (u16, u16) = joystick.xy();
                defmt::info!("JOYSTICK -> ID: {:?} | XY: {:?}", id, xy);
            }
            InputType::ASCII(input_ascii) => {
                let id: u8 = input_ascii.id();
                let c: char = input_ascii.char();
                defmt::info!("ASCII -> ID: {:?} | ASCII: {:?}", id, c);
            }
            InputType::Rotary(rotary) => {
                let id: u8 = rotary.id();
                let v: u16 = rotary.value();
                defmt::info!("ROTARY -> ID: {:?} | VALUE: {:?} ", id, v);
            }
            InputType::Button(button) => {
                let id: u8 = button.id();
                let state: &ButtonState = button.state();
                defmt::info!(
                    "BUTTON -> ID: {:?} | STATE: {} | DURATION: {}",
                    id,
                    state.pressed(),
                    state.duration()
                );
            }
        }
    }
}

#[repr(u8)]
pub enum DPad {
    Up = 0,
    Right = 1,
    Down = 2,
    Left = 3,
}

impl DPad {
    pub fn as_u8(&self) -> u8 {
        match self {
            DPad::Up => 0,
            DPad::Right => 1,
            DPad::Down => 2,
            DPad::Left => 3,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            DPad::Up => "Up",
            DPad::Right => "Right",
            DPad::Down => "Down",
            DPad::Left => "Left",
        }
    }
}

pub struct ButtonState {
    pressed: bool,
    duration: u16,
}

impl ButtonState {
    fn u64_duration_to_u16(duration: u64) -> u16 {
        if duration > u16::MAX as u64 {
            return u16::MAX;
        }
        duration as u16
    }
}

impl ButtonState {
    pub fn new(pressed: bool, duration: u64) -> Self {
        let duration = Self::u64_duration_to_u16(duration);
        Self { pressed, duration }
    }

    pub fn pressed(&self) -> bool {
        self.pressed
    }

    pub fn duration(&self) -> u16 {
        self.duration
    }
}

pub struct InputDPad {
    id: u8,
    dpad: DPad,
    state: ButtonState,
}

impl InputDPad {
    pub fn new(id: u8, dpad: DPad, state: ButtonState) -> Self {
        Self { id, dpad, state }
    }

    pub fn as_type(self) -> InputType {
        InputType::DPad(self)
    }

    pub fn from_buffer(buffer: &[u8; UDP_BUFFER]) -> Self {
        let id: u8 = *&buffer[0];
        let value: &u8 = &buffer[3];

        let dpad: DPad = match value {
            0 => DPad::Up,
            1 => DPad::Right,
            2 => DPad::Down,
            3 => DPad::Left,
            _ => panic!("Invalid DPad value: {}", value),
        };

        let pressed: &u8 = &buffer[4];
        let pressed: bool = u8_to_bool(*pressed);

        let duration: &[u8] = &buffer[5..=6];
        let duration: [u8; 2] = duration.try_into().unwrap();
        let duration: u16 = u16::from_be_bytes(duration);

        let state: ButtonState = ButtonState { pressed, duration };
        Self { id, dpad, state }
    }

    pub fn to_buffer(&self) -> [u8; UDP_BUFFER] {
        let id_be: u8 = self.id.to_be();
        let type_be: [u8; 2] = [0x00, 0x00];
        let dpad_be: u8 = self.dpad.as_u8();
        let pressed: u8 = self.state.pressed.into();
        let duration: [u8; 2] = split_u16(self.state.duration);

        let mut buffer: [u8; UDP_BUFFER] = [0; UDP_BUFFER];
        buffer[0] = id_be;
        buffer[1..=2].copy_from_slice(&type_be);
        buffer[3] = dpad_be;
        buffer[4] = pressed;
        buffer[5..=6].copy_from_slice(&duration);

        buffer
    }

    pub fn id(&self) -> u8 {
        self.id
    }

    pub fn dpad(&self) -> &DPad {
        &self.dpad
    }

    pub fn state(&self) -> &ButtonState {
        &self.state
    }
}

pub struct InputJoyStick {
    id: u8,
    x: u16,
    y: u16,
}

impl InputJoyStick {
    pub fn new(id: u8, x: u16, y: u16) -> Self {
        Self { id, x, y }
    }

    pub fn as_type(self) -> InputType {
        InputType::JoyStick(self)
    }

    pub fn from_buffer(buffer: &[u8; UDP_BUFFER]) -> Self {
        let id: u8 = *&buffer[0];
        let x_bytes: &[u8] = &buffer[3..=4];
        let y_bytes: &[u8] = &buffer[5..=6];

        let x_bytes: [u8; 2] = x_bytes.try_into().unwrap();
        let y_bytes: [u8; 2] = y_bytes.try_into().unwrap();

        let x: u16 = u16::from_be_bytes(x_bytes);
        let y: u16 = u16::from_be_bytes(y_bytes);

        Self { id, x, y }
    }

    pub fn to_buffer(&self) -> [u8; UDP_BUFFER] {
        let id_be: u8 = self.id.to_be();
        let type_be: [u8; 2] = [0x00, 0x01];
        let x_be: [u8; 2] = split_u16(self.x);
        let y_be: [u8; 2] = split_u16(self.y);

        let mut buffer: [u8; UDP_BUFFER] = [0; UDP_BUFFER];
        buffer[0] = id_be;
        buffer[1..=2].copy_from_slice(&type_be);
        buffer[3..=4].copy_from_slice(&x_be);
        buffer[5..=6].copy_from_slice(&y_be);

        buffer
    }

    pub fn id(&self) -> u8 {
        self.id
    }

    pub fn xy(&self) -> (u16, u16) {
        (self.x, self.y)
    }

    pub fn x(&self) -> u16 {
        self.x
    }

    pub fn y(&self) -> u16 {
        self.y
    }
}

pub struct InputASCII {
    id: u8,
    char: char,
}

impl InputASCII {
    pub fn new(id: u8, char: char) -> Self {
        Self { id, char }
    }

    pub fn as_type(self) -> InputType {
        InputType::ASCII(self)
    }

    pub fn from_buffer(buffer: &[u8; UDP_BUFFER]) -> Self {
        let id: u8 = *&buffer[0];
        let bytes: &u8 = &buffer[3];
        let bytes: u32 = *bytes as u32;
        let char: char = char::from_u32(bytes).unwrap();

        Self { id, char }
    }

    pub fn to_buffer(&self) -> [u8; UDP_BUFFER] {
        let id_be: u8 = self.id.to_be();
        let type_be: [u8; 2] = [0x00, 0x02];
        let char_be: u8 = self.char as u8;

        let mut buffer: [u8; UDP_BUFFER] = [0; UDP_BUFFER];
        buffer[0] = id_be;
        buffer[1..=2].copy_from_slice(&type_be);
        buffer[3] = char_be;

        buffer
    }

    pub fn id(&self) -> u8 {
        self.id
    }

    pub fn char(&self) -> char {
        self.char
    }
}

pub struct InputRotary {
    id: u8,
    value: u16,
}

impl InputRotary {
    pub fn new(id: u8, value: u16) -> Self {
        Self { id, value }
    }

    pub fn as_type(self) -> InputType {
        InputType::Rotary(self)
    }

    pub fn from_buffer(buffer: &[u8; UDP_BUFFER]) -> Self {
        let id: u8 = *&buffer[0];
        let bytes: &[u8] = &buffer[3..=4];
        let bytes: [u8; 2] = bytes.try_into().unwrap();
        let value: u16 = u16::from_be_bytes(bytes);

        Self { id, value }
    }

    pub fn to_buffer(&self) -> [u8; UDP_BUFFER] {
        let id_be: u8 = self.id.to_be();
        let type_be: [u8; 2] = [0x00, 0x03];
        let value_be: [u8; 2] = split_u16(self.value);

        let mut buffer: [u8; UDP_BUFFER] = [0; UDP_BUFFER];
        buffer[0] = id_be;
        buffer[1..=2].copy_from_slice(&type_be);
        buffer[3..=4].copy_from_slice(&value_be);

        buffer
    }

    pub fn id(&self) -> u8 {
        self.id
    }

    pub fn value(&self) -> u16 {
        self.value
    }
}

pub struct InputButton {
    id: u8,
    state: ButtonState,
}

impl InputButton {
    pub fn new(id: u8, state: ButtonState) -> Self {
        Self { id, state }
    }

    pub fn as_type(self) -> InputType {
        InputType::Button(self)
    }

    pub fn from_buffer(buffer: &[u8; UDP_BUFFER]) -> Self {
        let id: u8 = *&buffer[0];

        let pressed: &u8 = &buffer[3];
        let pressed: bool = u8_to_bool(*pressed);

        let duration: &[u8] = &buffer[4..=5];
        let duration: [u8; 2] = duration.try_into().unwrap();
        let duration: u16 = u16::from_be_bytes(duration);

        let state: ButtonState = ButtonState { pressed, duration };
        Self { id, state }
    }

    pub fn to_buffer(&self) -> [u8; UDP_BUFFER] {
        let id_be: u8 = self.id.to_be();
        let type_be: [u8; 2] = [0x00, 0x04];
        let pressed: u8 = self.state.pressed.into();
        let duration: [u8; 2] = split_u16(self.state.duration);

        let mut buffer: [u8; UDP_BUFFER] = [0; UDP_BUFFER];
        buffer[0] = id_be;
        buffer[1..=2].copy_from_slice(&type_be);
        buffer[3] = pressed;
        buffer[4..=5].copy_from_slice(&duration);

        buffer
    }

    pub fn id(&self) -> u8 {
        self.id
    }

    pub fn state(&self) -> &ButtonState {
        &self.state
    }
}
