use crate::input::InputJoyStick;

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
        let x: AdcGPIO = AdcGPIO::new(x_pin, Pull::Down);
        let y: AdcGPIO = AdcGPIO::new(y_pin, Pull::Down);
        Self { x, y }
    }
}

pub struct JoyStickDevice {
    adc: JoyStickAdc,
}

impl JoyStickDevice {
    pub fn new(adc: JoyStickAdc) -> Self {
        Self { adc }
    }

    pub async fn get_input(&mut self) -> Result<InputJoyStick, AdcError> {
        let x: u16 = self.adc.x.read().await?;
        let y: u16 = self.adc.y.read().await?;

        let joystick: InputJoyStick = InputJoyStick::new(0, x, y);
        Ok(joystick)
    }
}
