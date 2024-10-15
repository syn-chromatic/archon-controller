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

pub struct ExponentialMovingAverage {
    alpha: f32,
    ema: Option<f32>,
}

impl ExponentialMovingAverage {
    pub fn new(alpha: f32) -> Self {
        assert!(
            alpha >= 0.0 && alpha <= 1.0,
            "Alpha must be between 0 and 1"
        );
        ExponentialMovingAverage { alpha, ema: None }
    }

    pub fn from_period(period: usize) -> Self {
        assert!(period > 0, "Period must be greater than 0");
        let alpha: f32 = 2.0 / (period as f32 + 1.0);
        ExponentialMovingAverage::new(alpha)
    }

    pub fn update(&mut self, new_value: f32) -> f32 {
        self.ema = match self.ema {
            Some(current_ema) => Some(self.alpha * new_value + (1.0 - self.alpha) * current_ema),
            None => Some(new_value),
        };

        self.ema.unwrap()
    }

    pub fn value(&self) -> Option<f32> {
        self.ema
    }
}
