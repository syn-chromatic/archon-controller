#[allow(unused_imports)]
use crate::tests;

use crate::tasks::archon_collect;
use crate::tasks::archon_send;
use crate::tasks::wifi_connect;
use crate::transmitter::ArchonTransmitter;

use crate::devices::DevicesBuilder;

use crate::menu::menu_interface;

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

    // tests::joystick_test().await;
    // tests::rotary_test().await;
    // tests::dpad_test();
    // tests::button_test().await;
    // tests::test_device_layout().await;
    // tests::test_display_menu().await;

    let send_spawner: SendSpawner = spawner.make_send();
    let wifi_task: Task = Task::new(send_spawner, wifi_connect);

    defmt::info!("Initializing Startup Tasks..");
    let _ = wifi_task.start();
    // wifi_task.wait().await;

    let config_v4 = WIFIController::as_mut().get_config_v4();
    if let Some(config_v4) = config_v4 {
        let address = config_v4.address;
        defmt::info!("ADDRESS: {:?}", address);
    }

    menu_interface(send_spawner).await;

    // let discovery: MultiCastDiscovery = MultiCastDiscovery::new();
    // let _ = discovery.join().await;
    // let status: &DiscoveryStatus = discovery.start_discovery(&send_spawner).await.unwrap();
    // let establish: EstablishInformation = get_establish_information(&discovery, status).await;
    // discovery.stop_discovery().await;

    // let endpoint: ArchonEndpoint = establish.archon_endpoint();

    // ArchonTransmitter::read_lock().set_endpoint(endpoint);

    // let archon: _ = ArchonTransmitter::read_lock();
    // let mut layout: _ = archon.device_layout().lock();
    // DevicesBuilder::build(&mut layout).await;

    // defmt::info!("Archon Collecting..");
    // let archon_collect_task: Task = Task::new(send_spawner, archon_collect);
    // let _ = archon_collect_task.start();

    // defmt::info!("Archon Sending..");
    // let archon_send_task: Task = Task::new(send_spawner, archon_send);
    // let _ = archon_send_task.start();
}
