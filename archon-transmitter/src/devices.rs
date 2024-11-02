use archon_core::devices::button::ButtonConfiguration;
use archon_core::devices::button::ButtonDevice;
use archon_core::devices::dpad::DPadConfiguration;
use archon_core::devices::dpad::DPadDevice;
use archon_core::devices::dpad::DPadPins;
use archon_core::devices::joystick::JoyStickAdc;
use archon_core::devices::joystick::JoyStickConfiguration;
use archon_core::devices::joystick::JoyStickCoordinate;
use archon_core::devices::joystick::JoyStickDevice;
use archon_core::devices::joystick::JoyStickFilter;
use archon_core::devices::layout::DeviceLayout;
use archon_core::devices::polling::DevicePolling;
use archon_core::devices::rotary::RotaryAdc;
use archon_core::devices::rotary::RotaryConfiguration;
use archon_core::devices::rotary::RotaryDevice;
use archon_core::devices::rotary::RotaryFilter;

use embsys::drivers;
use embsys::exts::std;

use std::time::Duration;

use drivers::hardware::HWController;

pub struct DevicesBuilder;

impl DevicesBuilder {
    const BUTTON_BOUNCE: Duration = Duration::from_millis(5);
    const BUTTON_REPEAT: Duration = Duration::from_millis(100);
    const BUTTON_REPEAT_HOLD: Duration = Duration::from_millis(500);
    const ADC_POLLING: Duration = Duration::from_millis(10);

    pub fn dpad_device() -> DPadDevice {
        let pins: DPadPins = DPadPins::new(10, 11, 15, 14);
        let conf: DPadConfiguration = DPadConfiguration::new(
            Self::BUTTON_BOUNCE,
            Self::BUTTON_REPEAT,
            Self::BUTTON_REPEAT_HOLD,
        );
        let dpad_device: DPadDevice = DPadDevice::new(0, &pins, &conf);
        dpad_device
    }

    pub async fn joystick_device() -> JoyStickDevice {
        let x_pin = HWController::pac().PIN_26;
        let y_pin = HWController::pac().PIN_27;
        let adc: JoyStickAdc = JoyStickAdc::new(x_pin, y_pin);

        let origin: JoyStickCoordinate = JoyStickCoordinate::TopRight;
        let polling: DevicePolling = DevicePolling::new(Self::ADC_POLLING);
        let mut conf: JoyStickConfiguration = JoyStickConfiguration::new(origin, polling);

        let interpolation_filter: JoyStickFilter =
            JoyStickFilter::linear_interpolation(50, 4050, 50, 4050);
        let ema_filter: JoyStickFilter = JoyStickFilter::ema(5);
        conf.add_filter(interpolation_filter);
        conf.add_filter(ema_filter);

        let mut joystick_device: JoyStickDevice = JoyStickDevice::new(0, adc, conf);
        let _ = joystick_device.calibrate_center(5000).await;

        joystick_device
    }

    pub fn joystick_button_device() -> ButtonDevice {
        let conf: ButtonConfiguration = ButtonConfiguration::new(
            Self::BUTTON_BOUNCE,
            Self::BUTTON_REPEAT,
            Self::BUTTON_REPEAT_HOLD,
        );

        let button_device: ButtonDevice = ButtonDevice::new(0, 22, conf);
        button_device
    }

    pub fn l1_button_device() -> ButtonDevice {
        let conf: ButtonConfiguration = ButtonConfiguration::new(
            Self::BUTTON_BOUNCE,
            Self::BUTTON_REPEAT,
            Self::BUTTON_REPEAT_HOLD,
        );

        let button_device: ButtonDevice = ButtonDevice::new(1, 12, conf);
        button_device
    }

    pub async fn rotary_device() -> RotaryDevice {
        let v_pin = HWController::pac().PIN_28;
        let adc: RotaryAdc = RotaryAdc::new(v_pin);

        let polling: DevicePolling = DevicePolling::new(Self::ADC_POLLING);
        let mut conf: RotaryConfiguration = RotaryConfiguration::new(polling);

        let interpolation_filter: RotaryFilter = RotaryFilter::linear_interpolation(40, 4080);
        let ema_filter: RotaryFilter = RotaryFilter::ema(5);
        conf.add_filter(interpolation_filter);
        conf.add_filter(ema_filter);

        let rotary_device: RotaryDevice = RotaryDevice::new(0, adc, conf);
        rotary_device
    }

    pub async fn build(layout: &mut DeviceLayout) {
        let dpad_device: DPadDevice = Self::dpad_device();
        let joystick_device: JoyStickDevice = Self::joystick_device().await;
        let joystick_button_device: ButtonDevice = Self::joystick_button_device();
        let rotary_device: RotaryDevice = Self::rotary_device().await;
        let l1_button_device: ButtonDevice = Self::l1_button_device();

        layout.add_dpad(dpad_device);
        layout.add_joystick(joystick_device);
        layout.add_button(joystick_button_device);
        layout.add_rotary(rotary_device);
        layout.add_button(l1_button_device);
    }
}
