use super::polling::DevicePolling;

use crate::input::InputRotary;
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
}

impl RotaryFilter {
    pub fn ema(period: usize) -> Self {
        let v_ema: EMA<u16> = EMA::from_period(period);
        Self::EMA(v_ema)
    }
}

pub struct RotaryConfiguration {
    filter: RotaryFilter,
    polling: DevicePolling,
}

impl RotaryConfiguration {
    pub fn new(filter: RotaryFilter, polling: DevicePolling) -> Self {
        Self { filter, polling }
    }
}

pub struct RotaryDevice {
    adc: RotaryAdc,
    conf: RotaryConfiguration,
}

impl RotaryDevice {
    fn apply_filter(&mut self, v: &mut u16) {
        match &mut self.conf.filter {
            RotaryFilter::NoFilter => {}
            RotaryFilter::EMA(v_ema) => {
                *v = v_ema.update(*v);
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
        self.apply_filter(&mut v);

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

    pub async fn calibrate_center(&mut self, samples: usize) -> Result<(), AdcError> {
        for _ in 0..samples {
            let mut v: u16 = self.adc.v.read().await?;
            self.apply_filter(&mut v);
        }

        let mut v: u16 = self.adc.v.read().await?;
        self.apply_filter(&mut v);
        Ok(())
    }
}
