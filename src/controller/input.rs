#![allow(dead_code)]
#![allow(unused_variables)]

use crate::configuration::BUFFER;

// DATA REPRESENTATION
// [1-byte ID, 2-byte Type ID, X-byte Input Data]

#[repr(u16)]
pub enum InputType {
    /// 1-byte — [0x00] UP | [0x01] RIGHT | [0x02] DOWN | [0x03] LEFT
    DPad(InputDPad) = 0,
    /// 4-byte — [2-byte XAXIS, 2-byte YAXIS]
    JoyStick(InputJoyStick) = 1,
    /// 1-byte — [1-byte ASCII]
    ASCII(InputASCII) = 2,
    /// 2-byte — [2-byte ADC]
    Rotary(InputRotary) = 3,
}

impl InputType {
    pub fn from_buffer(buffer: &[u8; BUFFER]) -> InputType {
        let bytes: &[u8] = &buffer[1..=2];
        let bytes: [u8; 2] = bytes.try_into().unwrap();
        let type_id: u16 = u16::from_be_bytes(bytes);

        match type_id {
            0 => InputType::DPad(InputDPad::from_buffer(buffer)),
            1 => InputType::JoyStick(InputJoyStick::from_buffer(buffer)),
            2 => InputType::ASCII(InputASCII::from_buffer(buffer)),
            3 => InputType::Rotary(InputRotary::from_buffer(buffer)),
            _ => panic!("Unsupported InputType: {}", { type_id }),
        }
    }
}

fn split_u16(value: u16) -> [u8; 2] {
    let msb: u8 = (value >> 8) as u8;
    let lsb: u8 = (value & 0xFF) as u8;
    [msb, lsb]
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
}

pub struct InputDPad {
    id: u8,
    dpad: DPad,
}

impl InputDPad {
    pub fn from_buffer(buffer: &[u8; BUFFER]) -> Self {
        let id: u8 = *&buffer[0];
        let value: &u8 = &buffer[3];

        let dpad: DPad = match value {
            0 => DPad::Up,
            1 => DPad::Right,
            2 => DPad::Down,
            3 => DPad::Left,
            _ => panic!("Invalid DPad value: {}", value),
        };
        Self { id, dpad }
    }

    pub fn to_buffer(&self) -> [u8; 4] {
        let id_be: u8 = self.id.to_be();
        let type_be: [u8; 2] = [0x00, 0x00];
        let dpad_be: u8 = self.dpad.as_u8();

        let mut buffer: [u8; 4] = [0; 4];
        buffer[0] = id_be;
        buffer[1..=2].copy_from_slice(&type_be);
        buffer[3] = dpad_be;

        buffer
    }

    pub fn id(&self) -> u8 {
        self.id
    }

    pub fn dpad(&self) -> &DPad {
        &self.dpad
    }
}

pub struct InputJoyStick {
    id: u8,
    x: u16,
    y: u16,
}

impl InputJoyStick {
    pub fn from_buffer(buffer: &[u8; BUFFER]) -> Self {
        let id: u8 = *&buffer[0];
        let x_bytes: &[u8] = &buffer[3..=4];
        let y_bytes: &[u8] = &buffer[5..=6];

        let x_bytes: [u8; 2] = x_bytes.try_into().unwrap();
        let y_bytes: [u8; 2] = y_bytes.try_into().unwrap();

        let x: u16 = u16::from_be_bytes(x_bytes);
        let y: u16 = u16::from_be_bytes(y_bytes);

        Self { id, x, y }
    }

    pub fn to_buffer(&self) -> [u8; 4] {
        let id_be: u8 = self.id.to_be();
        let type_be: [u8; 2] = [0x00, 0x01];
        let x_be: [u8; 2] = split_u16(self.x);
        let y_be: [u8; 2] = split_u16(self.y);

        let mut buffer: [u8; 4] = [0; 4];
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
    pub fn from_buffer(buffer: &[u8; BUFFER]) -> Self {
        let id: u8 = *&buffer[0];
        let bytes: &u8 = &buffer[3];
        let bytes: u32 = *bytes as u32;
        let char: char = char::from_u32(bytes).unwrap();

        Self { id, char }
    }

    pub fn to_buffer(&self) -> [u8; 4] {
        let id_be: u8 = self.id.to_be();
        let type_be: [u8; 2] = [0x00, 0x02];
        let char_be: u8 = self.char as u8;

        let mut buffer: [u8; 4] = [0; 4];
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
    pub fn from_buffer(buffer: &[u8; BUFFER]) -> Self {
        let id: u8 = *&buffer[0];
        let bytes: &[u8] = &buffer[3..=4];
        let bytes: [u8; 2] = bytes.try_into().unwrap();
        let value: u16 = u16::from_be_bytes(bytes);

        Self { id, value }
    }

    pub fn to_buffer(&self) -> [u8; 4] {
        let id_be: u8 = self.id.to_be();
        let type_be: [u8; 2] = [0x00, 0x03];
        let value_be: [u8; 2] = split_u16(self.value);

        let mut buffer: [u8; 4] = [0; 4];
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
