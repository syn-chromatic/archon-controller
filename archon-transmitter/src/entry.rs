#[allow(unused_imports)]
use crate::tests;

use crate::tasks::archon_collect;
use crate::tasks::archon_send;
use crate::tasks::wifi_connect_static;
use crate::transmitter::ArchonTransmitter;

use crate::devices::create_dpad_device;
use crate::devices::create_joystick_button_device;
use crate::devices::create_joystick_device;
use crate::devices::create_l1_button_device;
use crate::devices::create_rotary_device;

use archon_core::devices::button::ButtonDevice;
use archon_core::devices::dpad::DPadDevice;
use archon_core::devices::joystick::JoyStickDevice;
use archon_core::devices::layout::DeviceLayout;
use archon_core::devices::rotary::RotaryDevice;
use archon_core::discovery::DiscoveryStatus;
use archon_core::discovery::EstablishInformation;
use archon_core::discovery::MultiCastDiscovery;
use archon_core::endpoint::ArchonEndpoint;

use embsys::crates::cortex_m_rt;
use embsys::crates::defmt;
use embsys::crates::embassy_executor;
use embsys::crates::embassy_time;
use embsys::drivers::hardware::WIFIController;
use embsys::exts::std;
use embsys::helpers;
use embsys::setup::SysInit;

use std::sync::Mutex;

use embassy_executor::SendSpawner;
use embassy_executor::Spawner;

use helpers::task_handler::Task;

async fn set_device_layout(layout: &Mutex<DeviceLayout>) {
    let dpad_device: DPadDevice = create_dpad_device();
    let joystick_device: JoyStickDevice = create_joystick_device().await;
    let joystick_button_device: ButtonDevice = create_joystick_button_device();
    let rotary_device: RotaryDevice = create_rotary_device().await;
    let l1_button_device: ButtonDevice = create_l1_button_device();

    layout.lock().add_dpad(dpad_device);
    layout.lock().add_joystick(joystick_device);
    layout.lock().add_button(joystick_button_device);
    layout.lock().add_rotary(rotary_device);
    layout.lock().add_button(l1_button_device);
}

async fn get_discovery_information(
    discovery: &MultiCastDiscovery,
    status: &DiscoveryStatus,
) -> EstablishInformation {
    loop {
        let state = status.state();
        let discovered = status.discovered();

        defmt::info!("State: {:?} | Discovered: {:?}", state, discovered);
        if !discovered.is_empty() {
            let info = discovered.last().unwrap().clone();
            let establish = discovery.connect(&info).await;
            if let Ok(establish) = establish {
                return establish;
            } else if let Err(error) = establish {
                defmt::info!("Error -> {:?}", error);
            }
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
    // tests::rotary_test().await;
    // tests::dpad_test();
    // tests::button_test().await;
    // tests::test_device_layout().await;

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
    let establish: EstablishInformation = get_discovery_information(&discovery, status).await;
    discovery.stop_discovery().await;

    let endpoint: ArchonEndpoint = establish.archon_endpoint();

    ArchonTransmitter::read_lock().set_endpoint(endpoint);

    set_device_layout(ArchonTransmitter::read_lock().device_layout()).await;

    defmt::info!("Archon Collecting..");
    let archon_collect_task: Task = Task::new(send_spawner, archon_collect);
    let _ = archon_collect_task.start();

    defmt::info!("Archon Sending..");
    let archon_send_task: Task = Task::new(send_spawner, archon_send);
    let _ = archon_send_task.start();
}
