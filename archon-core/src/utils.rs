pub fn u128_to_u16_max(value: u128) -> u16 {
    if value > u16::MAX as u128 {
        u16::MAX
    } else {
        value as u16
    }
}

pub fn split_u16(value: u16) -> [u8; 2] {
    let msb: u8 = (value >> 8) as u8;
    let lsb: u8 = (value & 0xFF) as u8;
    [msb, lsb]
}

pub fn u8_to_bool(value: u8) -> bool {
    if value == 1 {
        return true;
    } else if value == 0 {
        return false;
    }
    panic!("u8 value is not a boolean: {}", value);
}
