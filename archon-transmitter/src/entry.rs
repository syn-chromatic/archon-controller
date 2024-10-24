#[allow(unused_imports)]
use crate::tests;

use crate::tasks::archon_collect;
use crate::tasks::archon_send;
use crate::tasks::wifi_connect_static;
use crate::transmitter::ArchonTransmitter;

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
use archon_core::discovery::DiscoveryInformation;
use archon_core::discovery::DiscoveryStatus;
use archon_core::discovery::MultiCastDiscovery;
use archon_core::endpoint::ArchonEndpoint;

use embsys::crates::cortex_m_rt;
use embsys::crates::defmt;
use embsys::crates::embassy_executor;
use embsys::crates::embassy_time;
use embsys::drivers::hardware::HWController;
use embsys::drivers::hardware::WIFIController;
use embsys::exts::std;
use embsys::helpers;
use embsys::setup::SysInit;

use std::sync::Mutex;
use std::time::Duration as StdDuration;

use embassy_executor::SendSpawner;
use embassy_executor::Spawner;
use embassy_time::Duration;

use helpers::task_handler::Task;

fn create_dpad_device() -> DPadDevice {
    let bounce_interval: StdDuration = StdDuration::from_millis(10);
    let repeat_interval: StdDuration = StdDuration::from_millis(100);
    let repeat_hold: StdDuration = StdDuration::from_millis(500);

    let dpad_pins: DPadPins = DPadPins::new(10, 11, 15, 14);
    let dpad_conf: DPadConfiguration =
        DPadConfiguration::new(bounce_interval, repeat_interval, repeat_hold);
    let dpad_device: DPadDevice = DPadDevice::new(&dpad_pins, &dpad_conf);
    dpad_device
}

async fn create_joystick_device() -> JoyStickDevice {
    let x_pin = HWController::pac().PIN_26;
    let y_pin = HWController::pac().PIN_27;
    let joystick_adc: JoyStickAdc = JoyStickAdc::new(x_pin, y_pin);

    let joystick_origin: JoyStickCoordinate = JoyStickCoordinate::TopRight;
    let joystick_filter: JoyStickFilter = JoyStickFilter::ema(5);
    let joystick_polling: DevicePolling = DevicePolling::new(Duration::from_millis(10));

    let joystick_conf: JoyStickConfiguration =
        JoyStickConfiguration::new(joystick_origin, joystick_filter, joystick_polling);

    let mut joystick_device: JoyStickDevice = JoyStickDevice::new(joystick_adc, joystick_conf);
    let _ = joystick_device.calibrate_center(5000).await;

    joystick_device
}

async fn create_rotary_device() -> RotaryDevice {
    let v_pin = HWController::pac().PIN_28;
    let rotary_adc: RotaryAdc = RotaryAdc::new(v_pin);

    let rotary_filter: RotaryFilter = RotaryFilter::ema(5);
    let rotary_polling: DevicePolling = DevicePolling::new(Duration::from_millis(10));

    let rotary_conf: RotaryConfiguration = RotaryConfiguration::new(rotary_filter, rotary_polling);

    let mut rotary_device: RotaryDevice = RotaryDevice::new(rotary_adc, rotary_conf);
    let _ = rotary_device.calibrate_center(5000).await;

    rotary_device
}

async fn set_device_layout(layout: &Mutex<DeviceLayout>) {
    let dpad_device: DPadDevice = create_dpad_device();
    let joystick_device: JoyStickDevice = create_joystick_device().await;
    let rotary_device: RotaryDevice = create_rotary_device().await;
    layout.lock().add_dpad(dpad_device);
    layout.lock().add_joystick(joystick_device);
    layout.lock().add_rotary(rotary_device);
}

async fn get_discovery_information(status: &DiscoveryStatus) -> DiscoveryInformation {
    loop {
        let state = status.state();
        let discovered = status.discovered();

        defmt::info!("State: {:?} | Discovered: {:?}", state, discovered);
        if !discovered.is_empty() {
            return discovered.first().unwrap().clone();
        }

        embassy_time::Timer::after_millis(500).await;
    }
}

#[embassy_executor::main]
async fn entry(spawner: Spawner) {
    defmt::info!("Initializing System..");
    SysInit::hardware_controller();

    defmt::info!("Initializing WiFi Driver..");
    SysInit::wifi_controller(&spawner).await;

    WIFIController::control_mut().gpio_set(0, true).await;

    // tests::joystick_test().await;
    // tests::dpad_test();

    let send_spawner: SendSpawner = spawner.make_send();
    let wifi_task: Task = Task::new(send_spawner, wifi_connect_static);

    WIFIController::control_mut().gpio_set(0, false).await;

    defmt::info!("Initializing Startup Tasks..");
    let _ = wifi_task.start();
    wifi_task.wait().await;

    let config_v4 = WIFIController::borrow_mut().get_config_v4();
    if let Some(config_v4) = config_v4 {
        let address = config_v4.address;
        defmt::info!("ADDRESS: {:?}", address);
    }

    WIFIController::control_mut().gpio_set(0, true).await;

    let discovery: MultiCastDiscovery = MultiCastDiscovery::new();
    let _ = discovery.join().await;
    let status: &DiscoveryStatus = discovery.start_discovery(&send_spawner).await.unwrap();
    let disc_info: DiscoveryInformation = get_discovery_information(status).await;

    let endpoint: ArchonEndpoint = disc_info.endpoint();

    ArchonTransmitter::read_lock().set_endpoint(endpoint);

    set_device_layout(ArchonTransmitter::read_lock().device_layout()).await;

    defmt::info!("Archon Collecting..");
    let archon_collect_task: Task = Task::new(send_spawner, archon_collect);
    let _ = archon_collect_task.start();

    defmt::info!("Archon Sending..");
    let archon_send_task: Task = Task::new(send_spawner, archon_send);
    let _ = archon_send_task.start();
}
