use crate::configuration::BUFFER;

#[repr(u16)]
pub enum InputType {
    DPad = 0,
    JoyStick = 1,
    ASCII(InputASCII) = 2,
}

impl InputType {
    pub fn from_buffer(buffer: &[u8; BUFFER]) -> InputType {
        let bytes: &[u8] = &buffer[0..2];
        let bytes: [u8; 2] = bytes.try_into().expect("Slice with incorrect length");
        let value: u16 = u16::from_be_bytes(bytes);

        match value {
            0 => InputType::DPad,
            1 => InputType::JoyStick,
            2 => InputType::ASCII(InputASCII::from_buffer(buffer)),
            _ => panic!("Unsupported InputType: {}", { value }),
        }
    }
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
