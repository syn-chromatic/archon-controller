use num::cast::AsPrimitive;
use num::NumCast;
use num::Unsigned;

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

/// Exponential Moving Average
pub struct EMA<T> {
    alpha: f32,
    ema: Option<T>,
}

impl<T> EMA<T>
where
    T: Unsigned + Copy + AsPrimitive<f32> + NumCast,
{
    pub fn new(alpha: f32) -> Self {
        assert!(
            alpha >= 0.0 && alpha <= 1.0,
            "Alpha must be between 0.0 and 1.0"
        );
        EMA { alpha, ema: None }
    }

    pub fn from_period(period: usize) -> Self {
        assert!(period > 0, "Period must be greater than 0");
        let alpha: f32 = 2.0 / (period as f32 + 1.0);
        EMA::new(alpha)
    }

    pub fn update(&mut self, value: T) -> T {
        if let Some(ema) = self.ema {
            let ema_f32: f32 = self.alpha * value.as_() + (1.0 - self.alpha) * ema.as_();
            let ema: T = T::from(ema_f32).expect("Conversion failed");

            self.ema = Some(ema);
            return ema;
        }
        self.ema = Some(value);
        value
    }

    pub fn value(&self) -> Option<T> {
        self.ema
    }
}
