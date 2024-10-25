use super::polling::DevicePolling;

use crate::input::InputRotary;
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

pub struct RotaryAdc {
    v: AdcGPIO,
}

impl RotaryAdc {
    pub fn new(v_pin: impl Peripheral<P = impl AdcPin> + 'static) -> Self {
        let v: AdcGPIO = AdcGPIO::new(v_pin, Pull::None);
        Self { v }
    }
}

pub enum RotaryFilter {
    /// Disables Filter
    NoFilter,
    /// Exponential Moving Average
    EMA(EMA<u16>),
    LinearInterpolation(LinearInterpolationU12),
}

impl RotaryFilter {
    pub fn ema(period: usize) -> Self {
        let v_ema: EMA<u16> = EMA::from_period(period);
        Self::EMA(v_ema)
    }

    pub fn linear_interpolation(min_val: u16, max_val: u16) -> Self {
        let linear_interpolation: LinearInterpolationU12 =
            LinearInterpolationU12::new(min_val, max_val);
        Self::LinearInterpolation(linear_interpolation)
    }
}

pub struct RotaryConfiguration {
    filters: Vec<RotaryFilter>,
    polling: DevicePolling,
}

impl RotaryConfiguration {
    pub fn new(polling: DevicePolling) -> Self {
        let filters: Vec<RotaryFilter> = Vec::new();
        Self { filters, polling }
    }

    pub fn add_filter(&mut self, filter: RotaryFilter) {
        self.filters.push(filter);
    }
}

pub struct RotaryDevice {
    adc: RotaryAdc,
    conf: RotaryConfiguration,
}

impl RotaryDevice {
    fn apply_filters(&mut self, v: &mut u16) {
        for filter in &mut self.conf.filters {
            match filter {
                RotaryFilter::NoFilter => {}
                RotaryFilter::EMA(v_ema) => {
                    *v = v_ema.update(*v);
                }
                RotaryFilter::LinearInterpolation(inter) => {
                    *v = inter.interpolate(*v);
                }
            }
        }
    }
}

impl RotaryDevice {
    pub fn new(adc: RotaryAdc, conf: RotaryConfiguration) -> Self {
        Self { adc, conf }
    }

    pub async fn get_input(&mut self) -> Result<Option<InputRotary>, AdcError> {
        let mut v: u16 = self.adc.v.read().await?;
        self.apply_filters(&mut v);

        let v: u16 = ((v as f32 / 4095.0) * 10_000.0) as u16;

        if self.conf.polling.poll() {
            let rotary: InputRotary = InputRotary::new(0, v);
            return Ok(Some(rotary));
        }

        Ok(None)
    }

    pub async fn get_input_as_type(&mut self) -> Result<Option<InputType>, AdcError> {
        let rotary: Option<InputRotary> = self.get_input().await?;
        if let Some(rotary) = rotary {
            let input: InputType = InputType::Rotary(rotary);
            return Ok(Some(input));
        }
        Ok(None)
    }
}
