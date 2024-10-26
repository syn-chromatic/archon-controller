use super::polling::DevicePolling;

use crate::input::InputJoyStick;
use crate::input::InputType;
use crate::utils::LinearInterpolationU12;
use crate::utils::EMA;

use embsys::crates::embassy_rp;
use embsys::drivers::hardware;
use embsys::exts::std;

use std::vec::Vec;

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
    /// Linear Interpolation
    LinearInterpolation((LinearInterpolationU12, LinearInterpolationU12)),
}

impl JoyStickFilter {
    pub fn ema(period: usize) -> Self {
        let x_ema: EMA<u16> = EMA::from_period(period);
        let y_ema: EMA<u16> = EMA::from_period(period);

        Self::EMA((x_ema, y_ema))
    }

    pub fn linear_interpolation(x_min: u16, x_max: u16, y_min: u16, y_max: u16) -> Self {
        let inter_x: LinearInterpolationU12 = LinearInterpolationU12::new(x_min, x_max);
        let inter_y: LinearInterpolationU12 = LinearInterpolationU12::new(y_min, y_max);

        Self::LinearInterpolation((inter_x, inter_y))
    }
}

pub struct JoyStickCenter {
    x_center: u16,
    y_center: u16,
}

impl JoyStickCenter {
    pub fn new() -> Self {
        Self {
            x_center: 2048,
            y_center: 2048,
        }
    }

    pub fn set_center(&mut self, x: u16, y: u16) {
        self.x_center = x;
        self.y_center = y;
    }

    pub fn get_centered(&self, x: u16, y: u16) -> (u16, u16) {
        let x_centered: f32 = if x >= self.x_center {
            let p_range: f32 = 4095.0 - self.x_center as f32;
            let v_range: f32 = x as f32 - self.x_center as f32;
            let v: f32 = ((v_range / p_range) / 2.0) + 0.5;
            v * 10_000.0
        } else {
            let p_range = self.x_center as f32;
            let v_range = x as f32;
            let v: f32 = (v_range / p_range) / 2.0;
            v * 10_000.0
        };

        let y_centered: f32 = if y >= self.y_center {
            let p_range: f32 = 4095.0 - self.y_center as f32;
            let v_range: f32 = y as f32 - self.y_center as f32;
            let v: f32 = ((v_range / p_range) / 2.0) + 0.5;
            v * 10_000.0
        } else {
            let p_range = self.y_center as f32;
            let v_range = y as f32;
            let v: f32 = (v_range / p_range) / 2.0;
            v * 10_000.0
        };

        (x_centered as u16, y_centered as u16)
    }
}

pub struct JoyStickConfiguration {
    origin: JoyStickCoordinate,
    filters: Vec<JoyStickFilter>,
    center: JoyStickCenter,
    polling: DevicePolling,
}

impl JoyStickConfiguration {
    pub fn new(origin: JoyStickCoordinate, polling: DevicePolling) -> Self {
        let filters: Vec<JoyStickFilter> = Vec::new();
        let center: JoyStickCenter = JoyStickCenter::new();
        Self {
            origin,
            filters,
            center,
            polling,
        }
    }

    pub fn add_filter(&mut self, filter: JoyStickFilter) {
        self.filters.push(filter);
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
    id: u8,
    adc: JoyStickAdc,
    conf: JoyStickConfiguration,
}

impl JoyStickDevice {
    fn apply_filters(&mut self, x: &mut u16, y: &mut u16) {
        for filter in &mut self.conf.filters {
            match filter {
                JoyStickFilter::NoFilter => {}
                JoyStickFilter::EMA((x_ema, y_ema)) => {
                    *x = x_ema.update(*x);
                    *y = y_ema.update(*y);
                }
                JoyStickFilter::LinearInterpolation((inter_x, inter_y)) => {
                    *x = inter_x.interpolate(*x);
                    *y = inter_y.interpolate(*y);
                }
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
    pub fn new(id: u8, adc: JoyStickAdc, conf: JoyStickConfiguration) -> Self {
        Self { id, adc, conf }
    }

    pub async fn get_input(&mut self) -> Result<Option<InputJoyStick>, AdcError> {
        let mut x: u16 = self.adc.x.read().await?;
        let mut y: u16 = self.adc.y.read().await?;

        self.translate_origin(&mut x, &mut y);
        self.apply_filters(&mut x, &mut y);

        let (x, y) = self.conf.center.get_centered(x, y);

        if self.conf.polling.poll() {
            let joystick: InputJoyStick = InputJoyStick::new(self.id, x, y);
            return Ok(Some(joystick));
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
            self.apply_filters(&mut x, &mut y);
        }
        let mut x: u16 = self.adc.x.read().await?;
        let mut y: u16 = self.adc.y.read().await?;

        self.translate_origin(&mut x, &mut y);
        self.apply_filters(&mut x, &mut y);
        self.conf.center.set_center(x, y);
        Ok(())
    }
}
