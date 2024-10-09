use crate::configuration::BUFFER;

#[repr(u16)]
pub enum InputType {
    // 1-byte — [0x00] UP | [0x01] RIGHT | [0x02] DOWN | [0x03] LEFT
    DPad(InputDPad) = 0,
    // 4-byte — [2-byte XAXIS, 2-byte YAXIS]
    JoyStick(InputJoyStick) = 1,
    // 1-byte — [ASCII]
    ASCII(InputASCII) = 2,
}

impl InputType {
    pub fn from_buffer(buffer: &[u8; BUFFER]) -> InputType {
        let bytes: &[u8] = &buffer[0..2];
        let bytes: [u8; 2] = bytes.try_into().unwrap();
        let value: u16 = u16::from_be_bytes(bytes);

        match value {
            0 => InputType::DPad(InputDPad::from_buffer(buffer)),
            1 => InputType::JoyStick(InputJoyStick::from_buffer(buffer)),
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

impl InputDPad {
    pub fn from_buffer(buffer: &[u8; BUFFER]) -> Self {
        let data: &u8 = &buffer[2..3][0];

        let dpad: DPad = match data {
            0 => DPad::Up,
            1 => DPad::Right,
            2 => DPad::Down,
            3 => DPad::Left,
            _ => panic!("Invalid DPad Data: {}", data),
        };
        Self { dpad }
    }

    pub fn dpad(&self) -> &DPad {
        &self.dpad
    }
}

pub struct InputJoyStick {
    x: u16,
    y: u16,
}

impl InputJoyStick {
    pub fn from_buffer(buffer: &[u8; BUFFER]) -> Self {
        let x_data: &[u8] = &buffer[2..4];
        let y_data: &[u8] = &buffer[4..6];

        let x_data: [u8; 2] = x_data.try_into().unwrap();
        let y_data: [u8; 2] = y_data.try_into().unwrap();

        let x: u16 = u16::from_be_bytes(x_data);
        let y: u16 = u16::from_be_bytes(y_data);

        Self { x, y }
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
