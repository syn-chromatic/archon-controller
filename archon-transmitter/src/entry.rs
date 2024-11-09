#![allow(dead_code)]
#![allow(unused_imports)]

use crate::tasks::buffered_device_collect;
use crate::tasks::wifi_connect;
use crate::tests;
use crate::transmitter::ArchonTransmitter;

use crate::device::BufferedDeviceLayout;
use crate::device::DevicesBuilder;

use crate::interface::start_interface;

use archon_core::discovery::DiscoveryInformation;
use archon_core::discovery::DiscoveryStatus;
use archon_core::discovery::EstablishInformation;
use archon_core::discovery::MultiCastDiscovery;
use archon_core::endpoint::ArchonEndpoint;

use embsys::crates::cortex_m_rt;
use embsys::crates::defmt;
use embsys::crates::embassy_executor;
use embsys::crates::embassy_time;
use embsys::drivers::hardware::WIFIController;
use embsys::exts::non_std;
use embsys::exts::std;
use embsys::helpers;
use embsys::setup::SysInit;

use non_std::error::net::SocketError;
use std::vec::Vec;

use embassy_executor::SendSpawner;
use embassy_executor::Spawner;
use embassy_time::Timer;

use helpers::task_handler::Task;

async fn get_establish_information(
    discovery: &MultiCastDiscovery,
    status: &DiscoveryStatus,
) -> EstablishInformation {
    loop {
        let state: bool = status.state();
        let discovered: Vec<DiscoveryInformation> = status.discovered();

        defmt::info!("State: {:?} | Discovered: {:?}", state, discovered);
        if !discovered.is_empty() {
            if let Some(info) = discovered.last().cloned() {
                let establish: Result<EstablishInformation, SocketError> =
                    discovery.connect(&info).await;
                if let Ok(establish) = establish {
                    return establish;
                } else if let Err(error) = establish {
                    defmt::info!("Error -> {:?}", error);
                }
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

    defmt::info!("Initializing Devices Layout..");
    BufferedDeviceLayout::build_layout().await;

    let send_spawner: SendSpawner = spawner.make_send();

    defmt::info!("Initializing Wi-Fi Task..");
    let wifi_task: Task = Task::new(send_spawner, wifi_connect);
    let _ = wifi_task.start();

    let config_v4 = WIFIController::as_mut().get_config_v4();
    if let Some(config_v4) = config_v4 {
        let address = config_v4.address;
        defmt::info!("ADDRESS: {:?}", address);
    }

    defmt::info!("initializing Buffered Device Task..");
    let buffered_device_task: Task = Task::new(send_spawner, buffered_device_collect);
    let _ = buffered_device_task.start();

    defmt::info!("Starting Interface..");
    start_interface(send_spawner).await;
}
