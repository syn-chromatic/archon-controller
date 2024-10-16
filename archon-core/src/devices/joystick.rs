use super::polling::DevicePolling;

use crate::input::InputJoyStick;
use crate::input::InputType;
use crate::utils::EMA;

use embsys::crates::embassy_rp;
use embsys::drivers::hardware;

use embassy_rp::adc::AdcPin;
use embassy_rp::adc::Error as AdcError;
use embassy_rp::gpio::Pull;
use embassy_rp::Peripheral;

use hardware::AdcGPIO;
use hardware::AdcTrait;

pub struct JoyStickAdc {
    x: AdcGPIO,
    y: AdcGPIO,
}

impl JoyStickAdc {
    pub fn new(
        x_pin: impl Peripheral<P = impl AdcPin> + 'static,
        y_pin: impl Peripheral<P = impl AdcPin> + 'static,
    ) -> Self {
        let x: AdcGPIO = AdcGPIO::new(x_pin, Pull::None);
        let y: AdcGPIO = AdcGPIO::new(y_pin, Pull::None);
        Self { x, y }
    }
}

pub enum JoyStickCoordinate {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

pub enum JoyStickFilter {
    /// Disables Filter
    NoFilter,
    /// Exponential Moving Average
    EMA((EMA<u16>, EMA<u16>)),
}

impl JoyStickFilter {
    pub fn ema(period: usize) -> Self {
        let x_ema: EMA<u16> = EMA::from_period(period);
        let y_ema: EMA<u16> = EMA::from_period(period);

        Self::EMA((x_ema, y_ema))
    }
}

pub struct JoyStickOffset {
    x_offset: i16,
    y_offset: i16,
}

impl JoyStickOffset {
    fn clamp_u12(v: i16) -> u16 {
        if v < 0 {
            return 0;
        } else if v > 4095 {
            return 4095;
        }
        v as u16
    }
}

impl JoyStickOffset {
    pub fn new() -> Self {
        Self {
            x_offset: 0,
            y_offset: 0,
        }
    }

    pub fn center(&mut self, x: u16, y: u16) {
        self.x_offset = 2048 - x as i16;
        self.y_offset = 2048 - y as i16;
    }

    pub fn apply(&self, x: &mut u16, y: &mut u16) {
        let _x: i16 = *x as i16 + self.x_offset;
        let _y: i16 = *y as i16 + self.y_offset;
        *x = Self::clamp_u12(_x);
        *y = Self::clamp_u12(_y);
    }
}

pub struct JoyStickConfiguration {
    origin: JoyStickCoordinate,
    filter: JoyStickFilter,
    offset: JoyStickOffset,
    polling: DevicePolling,
}

impl JoyStickConfiguration {
    pub fn new(origin: JoyStickCoordinate, filter: JoyStickFilter, polling: DevicePolling) -> Self {
        let offset: JoyStickOffset = JoyStickOffset::new();
        Self {
            origin,
            filter,
            offset,
            polling,
        }
    }
}

pub struct JoyStickState {
    x: u16,
    y: u16,
}

impl JoyStickState {
    pub fn new() -> Self {
        Self { x: 0, y: 0 }
    }

    pub fn update(&mut self, x: u16, y: u16) -> Option<(u16, u16)> {
        if self.x != x || self.y != y {
            self.x = x;
            self.y = y;
            return Some((x, y));
        }
        None
    }
}

pub struct JoyStickDevice {
    adc: JoyStickAdc,
    state: JoyStickState,
    conf: JoyStickConfiguration,
}

impl JoyStickDevice {
    fn apply_filter(&mut self, x: &mut u16, y: &mut u16) {
        match &mut self.conf.filter {
            JoyStickFilter::NoFilter => {}
            JoyStickFilter::EMA((x_ema, y_ema)) => {
                *x = x_ema.update(*x);
                *y = y_ema.update(*y);
            }
        }
    }

    fn translate_origin(&mut self, x: &mut u16, y: &mut u16) {
        match &self.conf.origin {
            JoyStickCoordinate::TopLeft => {}
            JoyStickCoordinate::TopRight => *x = 4095 - *x,
            JoyStickCoordinate::BottomLeft => *y = 4095 - *y,
            JoyStickCoordinate::BottomRight => {
                *x = 4095 - *x;
                *y = 4095 - *y;
            }
        }
    }
}

impl JoyStickDevice {
    pub fn new(adc: JoyStickAdc, conf: JoyStickConfiguration) -> Self {
        let state: JoyStickState = JoyStickState::new();
        Self { adc, state, conf }
    }

    pub async fn get_input(&mut self) -> Result<Option<InputJoyStick>, AdcError> {
        let mut x: u16 = self.adc.x.read().await?;
        let mut y: u16 = self.adc.y.read().await?;

        self.translate_origin(&mut x, &mut y);
        self.conf.offset.apply(&mut x, &mut y);
        self.apply_filter(&mut x, &mut y);

        let state: Option<(u16, u16)> = self.state.update(x, y);
        if let Some(state) = state {
            if self.conf.polling.poll() {
                let joystick: InputJoyStick = InputJoyStick::new(0, state.0, state.1);
                return Ok(Some(joystick));
            }
        }

        Ok(None)
    }

    pub async fn get_input_as_type(&mut self) -> Result<Option<InputType>, AdcError> {
        let joystick: Option<InputJoyStick> = self.get_input().await?;
        if let Some(joystick) = joystick {
            let input: InputType = InputType::JoyStick(joystick);
            return Ok(Some(input));
        }
        Ok(None)
    }

    pub async fn calibrate_center(&mut self, samples: usize) -> Result<(), AdcError> {
        for _ in 0..samples {
            let mut x: u16 = self.adc.x.read().await?;
            let mut y: u16 = self.adc.y.read().await?;

            self.translate_origin(&mut x, &mut y);
            self.apply_filter(&mut x, &mut y);
        }
        let mut x: u16 = self.adc.x.read().await?;
        let mut y: u16 = self.adc.y.read().await?;

        self.translate_origin(&mut x, &mut y);
        self.apply_filter(&mut x, &mut y);
        self.conf.offset.center(x, y);
        Ok(())
    }
}
