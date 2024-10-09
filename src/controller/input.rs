use crate::configuration::BUFFER;

#[repr(u16)]
pub enum InputType {
    // 1-bytes — [0x00] UP | [0x01] RIGHT | [0x02] DOWN | [0x03] LEFT
    DPad(InputDPad) = 0,
    // 4-bytes — [2-byte XAXIS, 2-byte YAXIS]
    JoyStick(InputJoyStick) = 1,
    // 1-bytes — [ASCII]
    ASCII(InputASCII) = 2,
}

impl InputType {
    pub fn from_buffer(buffer: &[u8; BUFFER]) -> InputType {
        let bytes: &[u8] = &buffer[0..2];
        let bytes: [u8; 2] = bytes.try_into().expect("Slice with incorrect length");
        let value: u16 = u16::from_be_bytes(bytes);

        match value {
            0 => unimplemented!("InputType Unimplemented"),
            1 => unimplemented!("InputType Unimplemented"),
            2 => InputType::ASCII(InputASCII::from_buffer(buffer)),
            _ => panic!("Unsupported InputType: {}", { value }),
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

pub struct InputDPad {
    dpad: DPad,
}

pub struct InputJoyStick {
    x: u16,
    y: u16,
}

pub struct InputASCII {
    char: char,
}

impl InputASCII {
    pub fn from_buffer(buffer: &[u8; BUFFER]) -> Self {
        let data: &u8 = &buffer[2..3][0];
        let data: u32 = *data as u32;
        let char: char = char::from_u32(data).unwrap();

        Self { char }
    }

    pub fn char(&self) -> char {
        self.char
    }
}
